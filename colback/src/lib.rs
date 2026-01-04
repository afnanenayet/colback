//! # colback
//!
//! Column backed lists of structs.
//!
//! # Synopsis
//!
//! This crate provides procedural macros that generate code to extract data from dataframes so
//! that their column data can be used as a view into a proxy struct. You want to keep contiguous
//! columnar data, but you want to be able to write normal code with type-safety and minimal
//! boilerplate that is also performant.
//!
//! # What it does
//!
//! This will first create some proxy view and reference structs based on a struct that you use a
//! derive macro on. The derive macro then generates some code that will handle extracting the
//! chunked arrays backing the dataframe columns, as well as proxy/view types for each row. These
//! proxy types are populated using data from the dataframe, so each instance of a proxy struct
//! corresponds directly to a row.
//!
//! # Example
//!
//! ```rust
//! use colback::ColbackView;
//! use polars::df;
//!
//! #[derive(ColbackView)]
//! struct MyRow {
//!     col_a: u32,
//!     col_b: bool,
//! }
//! let df = df!["col_a" => [0u32, 1u32], "col_b" => [true, false]].unwrap();
//! let row_view = MyRow::view(&df).unwrap();
//! let row_ref = row_view.get(0).unwrap();
//! assert_eq!(row_ref.col_a, 0);
//! assert_eq!(row_ref.col_b, true);
//! ```

// Trick to allow for codegen within the same crate. This was also required to get the doctest
// working.
extern crate self as colback;

pub use colback_derive::ColbackView;
use polars::{frame::DataFrame, prelude::DataType};
use thiserror::Error;

/// Errors that can arise when trying to extract a dataframe to a row view.
#[derive(Debug, Error)]
pub enum ColbackError {
    /// Error for when the derive macro is called on a struct without named fields.
    #[error("expected a struct with named fields")]
    UnnamedFields,

    /// When the dataframe is missing a required column that was specified in the struct.
    #[error("missing required column(s): {0:?}")]
    MissingColumn(String),

    #[error("column {col} has wrong dtype: expected {expected:?}, got {actual:?}")]
    WrongDtype {
        col: String,
        expected: DataType,
        actual: DataType,
    },

    /// Thrown if the dataframe has a null value and the null handling policy is to error out.
    #[error("null values encountered in non-nullable column {col} at row {idx}")]
    InvalidNull {
        /// Name of the column with the null value
        col: String,
        // Usually indices are u32 but a feature can enable u64 columns, we're willing to take the
        // hit for an error since we don't need to optimize the sad path.
        /// Index where the null was encountered
        idx: usize,
    },
}

/// Convenience alias for results from this crate.
pub type Result<T> = std::result::Result<T, ColbackError>;

/// Trait for a struct that contains a reference to a row of a dataframe.
///
/// This is typically implemented by the crate's derive macros. This is implemented on a fully
/// realized struct and defines the associated generated row reference struct types.
pub trait ColbackView: Sized {
    /// Stores the underlying data required for row reference structs.
    ///
    /// This has a reference to the dataframe and the extracted column chunks and is used to
    /// generate the row reference proxies.
    type View<'a>
    where
        Self: 'a;

    /// The proxy class that represents a row.
    type RowRef<'a>
    where
        Self: 'a;

    /// Create a view struct for a given dataframe.
    ///
    /// The view struct can be used to generate row reference proxy structs.
    ///
    /// # Errors
    ///
    /// This may throw an error if the dataframe is missing data, has nulls (depending on the null
    /// handling policy), or if there are dtype mismatches. See [ColbackError] for more details.
    fn view(df: &DataFrame) -> Result<Self::View<'_>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use colback_derive::ColbackView;
    use polars::df;

    #[test]
    fn ui_pass() {
        let t = trybuild::TestCases::new();
        t.pass("tests/ui/pass_*.rs");
    }

    #[test]
    fn ui_fail() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/fail_*.rs");
    }

    #[test]
    fn test_extract_struct() {
        #[derive(ColbackView, Eq, PartialEq)]
        struct SomeStruct {
            row_a: u32,
            row_b: bool,
        }

        let df = df! [
            "row_a" => [0u32, 1u32],
            "row_b" => [true, false],
        ]
        .unwrap();

        let x = SomeStruct::view(&df).unwrap();

        {
            let row_ref = x.get(0).unwrap();
            assert_eq!(row_ref.row_a, 0);
            assert!(row_ref.row_b);
        }
        {
            let row_ref = x.get(1).unwrap();
            assert_eq!(row_ref.row_a, 1);
            assert!(!row_ref.row_b);
        }
    }
}
