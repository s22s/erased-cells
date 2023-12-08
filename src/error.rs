/*
 * Copyright (c) 2023. Astraea, Inc. All rights reserved.
 */

use crate::CellType;
use thiserror::Error as ThisError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(ThisError, Debug, PartialEq)]
pub enum Error {
    #[error("Invalid narrowing from cell type {src} to {dst}")]
    NarrowingError { src: CellType, dst: CellType },
    #[error("Expected a value but received `None`: {0}")]
    ExpectedError(String),
    #[error("Unable to parse {0} as a {1}")]
    ParseError(String, &'static str),
}
