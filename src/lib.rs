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
//! # Example
//!
//! ```rust
//! use erased_cells::{CellBuffer, CellType, CellValue};
//! // Fill a buffer with the `u8` numbers `0..=8`.
//! let buf1 = CellBuffer::fill_with(9, |i| i as u8);
//!
//! // `u8` maps to `CellType::UInt8`
//! assert_eq!(buf1.cell_type(), CellType::UInt8);
//!
//! // A fetching values maintains its CellType through a CellValue.
//! let val: CellValue = buf1.get(3);
//! assert_eq!(val, CellValue::UInt8(3));
//! let (min, max): (CellValue, CellValue) = buf1.minmax();
//! assert_eq!((min, max), (CellValue::UInt8(0), CellValue::UInt8(8)));
//!
//! // Basic math ops work on CellValues. Primitives can be converted to CellValues with `into`.
//! // Math ops coerce to floating point values.
//! assert_eq!(((max - min + 1.into()) / 2.into()), 4.5.into());
//!
//! // Fill another buffer with the `f32` numbers `8..=0`.
//! let buf2 = CellBuffer::fill_with(9, |i| 8f32 - i as f32);
//! assert_eq!(buf2.cell_type(), CellType::Float32);
//! assert_eq!(
//! buf2.minmax(),
//! (CellValue::Float32(0.0), CellValue::Float32(8.0))
//! );
//!
//! // Basic math ops also work on CellBuffers, applied element-wise.
//! let diff = buf2 - buf1;
//! assert_eq!(diff.minmax(), ((-8).into(), 8.into()));
//! ```
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
