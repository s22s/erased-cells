/*
 * Copyright (c) 2023. Astraea, Inc. All rights reserved.
 */

use crate::CellType;
use thiserror::Error as ThisError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Invalid narrowing from cell type {src} to {dst}")]
    NarrowingError { src: CellType, dst: CellType },
    #[error("Expected a value but received `None`: {0}")]
    ExpectedError(String),
    #[error("Unable to parse {0} as a {1}")]
    ParseError(String, &'static str),
}

// pub trait ExpectOr {
//     type Output;
//     fn expect_else<F>(self, msg: F) -> Result<Self::Output>
//     where
//         F: FnOnce() -> String;
// }
//
// impl<T> ExpectOr for Option<T> {
//     type Output = T;
//     fn expect_else<F>(self, msg: F) -> Result<T>
//     where
//         F: FnOnce() -> String,
//     {
//         match self {
//             None => Err(Error::ExpectedError(msg())),
//             Some(v) => Ok(v),
//         }
//     }
// }
