use quote::quote;

pub struct TypeMap {
    pub expected_dtype: proc_macro2::TokenStream,
    pub accessor: syn::Ident,
    pub chunked_ty: proc_macro2::TokenStream,
    pub row_value_ty: proc_macro2::TokenStream,
    pub get_value_expr: proc_macro2::TokenStream,
}

pub fn option_inner(ty: &syn::Type) -> (bool, syn::Type) {
    // MVP: detect Option<T> only for the canonical path Option<...>
    if let syn::Type::Path(tp) = ty
        && tp.qself.is_none()
        && tp.path.segments.len() == 1
        && tp.path.segments[0].ident == "Option"
        && let syn::PathArguments::AngleBracketed(ref args) = tp.path.segments[0].arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return (true, inner.clone());
    }
    (false, ty.clone())
}

/// Map primitive types to polars dtypes
///
/// This handles boilerplate for a match statement that constructs a type map definition based on
/// the string value of primitive types.
macro_rules! map_prim {
    (
        $ident_str:expr,
        $get_value_expr:expr,
        $( $rust:literal => {
            dtype: $dtype:ident,
            accessor: $accessor:literal,
            chunked: $chunked:ident,
            row_ty: $row_ty:tt $( $row_ty_tail:tt )*
        } ),* $(,)?
    ) => {{
        match $ident_str {
            $(
                $rust => Some(TypeMap {
                    expected_dtype: quote!(::polars::prelude::DataType::$dtype),
                    accessor: syn::Ident::new($accessor, proc_macro2::Span::call_site()),
                    chunked_ty: quote!(::polars::prelude::$chunked),
                    row_value_ty: quote!($row_ty $( $row_ty_tail )*),
                    get_value_expr: $get_value_expr,
                }),
            )*
            _ => None,
        }
    }};
}

/// Map primitive Rust types to polars dtypes for fields of a struct.
///
/// This *does not* handle `Option<T>` types, this is only meant for the inner types.
pub fn map_type(col_ident: &syn::Ident, ty: &syn::Type) -> Option<TypeMap> {
    let ident = match ty {
        syn::Type::Path(tp) if tp.qself.is_none() && tp.path.segments.len() == 1 => {
            tp.path.segments[0].ident.to_string()
        }
        _ => return None,
    };

    let get_value_expr = quote!(self.#col_ident.get(idx));
    // TODO: determine best way to handle categoricals
    map_prim!(
        ident.as_str(),
        get_value_expr,
        "u8" => { dtype: UInt8, accessor: "u8", chunked: UInt8Chunked, row_ty: u8 },
        "u16" => { dtype: UInt16, accessor: "u16", chunked: UInt16Chunked, row_ty: u16 },
        "u32" => { dtype: UInt32, accessor: "u32", chunked: UInt32Chunked, row_ty: u32 },
        "u64" => { dtype: UInt64, accessor: "u64", chunked: UInt63Chunked, row_ty: u64 },
        "i32" => { dtype: Int32,  accessor: "i32", chunked: Int32Chunked,  row_ty: i32 },
        "i64" => { dtype: Int64,  accessor: "i64", chunked: Int64Chunked,  row_ty: i64 },
        "f32" => { dtype: Float32, accessor: "f32", chunked: Float32Chunked, row_ty: f32 },
        "f64" => { dtype: Float64, accessor: "f64", chunked: Float64Chunked, row_ty: f64 },
        "bool" => { dtype: Boolean, accessor: "bool", chunked: BooleanChunked, row_ty: bool },
        "String" => { dtype: String, accessor: "str", chunked: StringChunked, row_ty: &'a str },
    )
}
