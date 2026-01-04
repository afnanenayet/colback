mod type_helpers;

use crate::type_helpers::{map_type, option_inner};
use darling::FromField;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Get the runtime path of the colback crate.
///
/// This is robust to crates being renamed when users declare the dependency in cargo.
fn runtime_path() -> proc_macro2::TokenStream {
    match crate_name("colback-core") {
        Ok(FoundCrate::Itself) => quote!(crate), // only if derive is inside same crate (rare)
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(::#ident)
        }
        Err(_) => quote!(::colback), // fallback; emits a good error later if missing
    }
}

/// Field attributes specifying how a column value should map to a row view.
#[derive(Debug, FromField)]
#[darling(attributes(polars))]
struct ColbackFieldOpts {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    /// The name of the corresponding column in the dataframe.
    ///
    /// If this is *not* supplied then the name will be set to the name of the struct field.
    #[darling(default)]
    name: Option<String>,

    /// Can be one of:
    ///
    /// - "error": Will raise an error when trying to extract the dataframe if any values are null
    /// - "option": The derived view struct will have use an optional type so that some values can
    ///    be null. This is the only allowed value if the original struct has a field set to
    ///    optional.
    /// - "default": Null row values will be replaced by some default value.
    #[darling(default)]
    null: Option<String>,

    /// The default value to use for null row values.
    ///
    /// This is required if "default" is selected for the null handling policy. Setting this value
    /// is an error if the `null` field is set to anything besides "default".
    #[darling(default)]
    default: Option<syn::Expr>,
}

#[proc_macro_error]
#[proc_macro_derive(ColbackView, attributes(polars))]
pub fn derive_colback_view(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let rt = runtime_path();
    let struct_name = input.ident;

    let fields = match input.data {
        Data::Struct(ref s) => match s.fields {
            Fields::Named(ref named) => named.named.iter().collect::<Vec<_>>(),
            _ => abort!(
                struct_name,
                "ColbackView only supports structs with named fields"
            ),
        },
        _ => abort!(struct_name, "ColbackView can only be derived for structs"),
    };

    let mut parsed = Vec::new();
    for f in fields {
        let opts = match ColbackFieldOpts::from_field(f) {
            Ok(v) => v,
            Err(e) => abort!(struct_name, "invalid #[polars(...)] on field: {}", e),
        };

        let ident = opts.ident.clone().unwrap();
        let col_name = opts.name.clone().unwrap_or_else(|| ident.to_string());

        parsed.push((
            ident,
            opts.ty.clone(),
            col_name,
            opts.null.clone(),
            opts.default.clone(),
        ));
    }

    // Generated types: <StructName>View<'a> and <StructName>RowRef<'a>
    let view_name = format_ident!("{}View", struct_name);
    let rowref_name = format_ident!("{}RowRef", struct_name);

    // For each field, generate:
    // - a member in View<'a> holding a typed ChunkedArray reference
    // - validation + extraction in try_new
    // - row materialization in get_row (using get(idx))
    let mut view_members = Vec::new();
    let mut extract_stmts = Vec::new();
    let mut row_members = Vec::new();
    let mut row_build = Vec::new();
    let mut view_ctor_idents: Vec<syn::Ident> = Vec::new();
    let mut row_ctor_idents: Vec<syn::Ident> = Vec::new();

    for (ident, ty, col_name, null_policy, default_expr) in parsed {
        // Detect Option<T>
        let (is_option, inner_ty) = option_inner(&ty);

        // Map Rust type to:
        // - Polars DataType for validation
        // - Series accessor (u32(), i64(), f64(), bool(), str())
        // - ChunkedArray type in View
        // - row getter expression
        let map = match map_type(&ident, &inner_ty) {
            Some(m) => m,
            None => abort!(
                ident,
                "unsupported field type for ColbackView; add a mapping for this type"
            ),
        };

        let view_field_ty = map.chunked_ty;
        let expected_dtype = map.expected_dtype;
        let accessor = map.accessor;
        let row_value_ty = map.row_value_ty;
        let get_value = map.get_value_expr;

        let policy = null_policy.as_deref().unwrap_or("error");
        match (policy, is_option, &default_expr) {
            ("option", false, _) => abort!(
                ident,
                "null=\"option\" requires the field type to be Option<T>"
            ),
            ("error", true, _) => abort!(ident, "Option<T> fields must use null='option'"),
            ("default", _, None) => abort!(
                ident,
                "null='default' requires #[polars(default = ...)] to be set"
            ),
            _ => (),
        };

        // View member
        view_members.push(quote! {
            #ident: &'a #view_field_ty
        });

        let col_var_name = format_ident!("{}_col", col_name);

        // Extraction + dtype check
        // TODO: allow type casting here, with warnings
        extract_stmts.push(quote! {
            let #col_var_name = df.column(#col_name)
                .map_err(|_| #rt::ColbackError::MissingColumn(#col_name.to_string()))?;
            if #col_var_name.dtype() != &#expected_dtype {
                return Err(#rt::ColbackError::WrongDtype {
                    col: #col_name.to_string(),
                    expected: #expected_dtype.clone(),
                    actual: #col_var_name.dtype().clone(),
                });
            }
            let #ident = #col_var_name.#accessor().expect("dtype checked above");
        });

        // RowRef member type (borrowed)
        if is_option {
            row_members.push(quote! { pub #ident: Option<#row_value_ty> });
            row_build.push(quote! {
                let #ident = #get_value;
            });
        } else if policy == "default" {
            let def = default_expr.unwrap();
            row_members.push(quote! { pub #ident: #row_value_ty });
            row_build.push(quote! {
                let #ident = match #get_value {
                    Some(v) => v,
                    None => #def,
                };
            });
        } else {
            // error on null
            row_members.push(quote! { pub #ident: #row_value_ty });
            row_build.push(quote! {
                let #ident = #get_value.ok_or_else(|| #rt::ColbackError::InvalidNull{ col: #col_name.to_string(), idx })?;
            });
        }
        view_ctor_idents.push(ident.clone());
        row_ctor_idents.push(ident.clone());
    }

    let expanded: proc_macro2::TokenStream = quote! {
        pub struct #view_name<'a> {
            df: &'a ::polars::prelude::DataFrame,
            #(#view_members),*
        }

        pub struct #rowref_name<'a> {
            pub _data: ::std::marker::PhantomData<&'a ()>,
            #(#row_members),*
        }

        impl<'a> #view_name<'a> {
            pub fn df(&self) -> &'a ::polars::prelude::DataFrame {
                self.df
            }

            pub fn len(&self) -> usize {
                self.df.height()
            }

            pub fn get(&'a self, idx: usize) -> #rt::Result<#rowref_name<'a>> {
                #(#row_build)*
                Ok(#rowref_name { _data: Default::default(), #(#row_ctor_idents),* })
            }

            pub fn iter(&'a self) -> impl Iterator<Item = #rt::Result<#rowref_name<'a>>> + 'a {
                (0..self.len()).map(|i| self.get(i))
            }
        }

        impl #rt::ColbackView for #struct_name {
            type View<'a> = #view_name<'a> where Self: 'a;
            type RowRef<'a> = #rowref_name<'a> where Self: 'a;

            fn view<'a>(df: &'a ::polars::prelude::DataFrame) -> #rt::Result<Self::View<'a>> {
                #(#extract_stmts)*

                Ok(#view_name {
                    df,
                    #(#view_ctor_idents),*
                })
            }
        }

    };
    expanded.into()
}
