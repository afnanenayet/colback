pub use colback_derive::ColbackView;
use polars::{frame::DataFrame, prelude::DataType};
use thiserror::Error;

/// Errors that can arise when trying to extract a dataframe to a row view.
#[derive(Debug, Error)]
pub enum ColbackError {
    #[error("expected a struct with named fields")]
    UnnamedFields,

    #[error("missing required column(s): {0:?}")]
    MissingColumn(String),

    #[error("column {col} has wrong dtype: expected {expected:?}, got {actual:?}")]
    WrongDtype {
        col: String,
        expected: DataType,
        actual: DataType,
    },

    #[error("null values encountered in non-nullable column {col} at row {idx}")]
    InvalidNull {
        col: String,
        // Usually indices are u32 but a feature can enable u64 columns, we're willing to take the
        // hit for an error since we don't need to optimize the sad path.
        idx: usize,
    },
}

/// Convenience alias
pub type Result<T> = std::result::Result<T, ColbackError>;

/// Trait for a struct that contains a reference to a row of a dataframe.
///
/// This is typically implemented by the crate's derive macros. This is implemented on a fully
/// realized struct and defines the associated generated row reference struct types.
pub trait ColbackView: Sized {
    type View<'a>
    where
        Self: 'a;

    type RowRef<'a>
    where
        Self: 'a;

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
            assert_eq!(row_ref.row_b, true);
        }
        {
            let row_ref = x.get(1).unwrap();
            assert_eq!(row_ref.row_a, 1);
            assert_eq!(row_ref.row_b, false);
        }
    }
}
