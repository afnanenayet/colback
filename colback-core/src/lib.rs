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

    fn view<'a>(df: &'a DataFrame) -> Result<Self::View<'a>>;
}
