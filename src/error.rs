//! Crate-wide `Result`/`Error` types.

use crate::CellType;
use thiserror::Error as ThisError;

#[cfg(feature = "gdal")]
use gdal::errors::GdalError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Enumeration of error kinds
#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Invalid narrowing from cell type {src} to {dst}")]
    NarrowingError { src: CellType, dst: CellType },
    #[error("Unsupported cell type {0}")]
    UnsupportedCellTypeError(String),
    #[error("Expected a value but received `None`: {0}")]
    ExpectedError(String),
    #[error("Unable to parse {0} as a {1}")]
    ParseError(String, &'static str),
    #[cfg(feature = "gdal")]
    #[error(transparent)]
    GdalError(#[from] GdalError),
}
