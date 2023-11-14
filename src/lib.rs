/*
 * Copyright (c) 2023. Astraea, Inc. All rights reserved.
 */

//! Encoding and manipulation of runtime-dynamic cell values.
//!
//! # Synopsis
//!
//! This crate enables the manipulation of heterogeneous values and buffers of Rust primitive numeric types.
//! It is useful in cases where the numeric encoding is either not known at compile-time, or when
//! multiple encodings are in use yet need to be treated in a homogenous way. The types are
//! normalized using discriminated unions (`enums`).
//!
//! There are three core enums:
//!
//! * [`CellType`]: An enumeration of each supported primitive type.
//! * [`CellValue`]: A scalar primitive value stored as a [`CellType`] associated variant.
//! * [`CellBuffer`]: A `Vec<_>` of primitive values stored as a [`CellType`] associated variant.
//!
//!

mod buffer;
mod ctype;
mod encoding;
pub mod error;
mod value;

pub use buffer::ops::*;
pub use buffer::*;
pub use ctype::*;
pub use encoding::*;
pub use value::ops::*;
pub use value::*;

/// A [callback style](https://danielkeep.github.io/tlborm/book/pat-callbacks.html)
/// macro used to construct various implementations in this crate.
macro_rules! with_ct {
    ($callback:ident) => {
        $callback! {
            (UInt8, u8),
            (UInt16, u16),
            (UInt32, u32),
            (UInt64, u64),
            (Int8, i8),
            (Int16, i16),
            (Int32, i32),
            (Int64, i64),
            (Float32, f32),
            (Float64, f64)
        }
    };
}
pub(crate) use with_ct;
