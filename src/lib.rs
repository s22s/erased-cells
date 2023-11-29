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
//! use erased_cells::{CellBuffer, CellType, CellValue, BufferOps};
//! // Fill a buffer with the `u8` numbers `0..=8`.
//! let buf1 = CellBuffer::fill_with(9, |i| i as u8);
//!
//! // `u8` maps to `CellType::UInt8`
//! assert_eq!(buf1.cell_type(), CellType::UInt8);
//!
//! // A fetching values maintains its CellType through a CellValue.
//! let val: CellValue = buf1.get(3);
//! assert_eq!(val, CellValue::UInt8(3));
//! let (min, max): (CellValue, CellValue) = buf1.min_max();
//! assert_eq!((min, max), (CellValue::UInt8(0), CellValue::UInt8(8)));
//!
//! // Basic math ops work on CellValues. Primitives can be converted to CellValues with `into`.
//! // Math ops coerce to floating point values.
//! assert_eq!(((max - min + 1) / 2), 4.5.into());
//!
//! // Fill another buffer with the `f32` numbers `8..=0`.
//! let buf2 = CellBuffer::fill_with(9, |i| 8f32 - i as f32);
//! assert_eq!(buf2.cell_type(), CellType::Float32);
//! assert_eq!(
//! buf2.min_max(),
//! (CellValue::Float32(0.0), CellValue::Float32(8.0))
//! );
//!
//! // Basic math ops also work on CellBuffers, applied element-wise.
//! let diff = buf2 - buf1;
//! assert_eq!(diff.min_max(), ((-8).into(), 8.into()));
//! ```
mod buffer;
mod ctype;
mod encoding;
pub mod error;
#[cfg(feature = "masked")]
mod masked;
mod value;

pub use buffer::ops::*;
pub use buffer::*;
pub use ctype::*;
pub use encoding::*;
#[cfg(feature = "masked")]
pub use masked::*;
use std::fmt::{Debug, Formatter};
pub use value::ops::*;
pub use value::*;

/// A [callback style](https://danielkeep.github.io/tlborm/book/pat-callbacks.html)
/// macro used to construct various implementations covering all [`CellType`]s.
///
/// It calls the passed identifier as a macro with two parameters:
/// * the cell type id (e.g. `UInt8`),
/// * the cell type primitive (e.g. `u8`).
///
/// # Example
/// ```rust
/// use erased_cells::{with_ct, CellType};
/// fn primitive_name(ct: CellType) -> &'static str {
///     macro_rules! primitive_name {
///        ($(($id:ident, $p:ident)),*) => {
///             match ct {
///                 $(CellType::$id => stringify!($p),)*
///             }
///        };
///     }
///     with_ct!(primitive_name)
/// }
///
/// assert_eq!(primitive_name(CellType::Float32), "f32");
/// ```
#[macro_export]
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

pub trait BufferOps {
    /// Construct a [`CellBuffer`] from a `Vec<T>`.
    fn from_vec<T: CellEncoding>(data: Vec<T>) -> Self;

    /// Construct a [`CellBuffer`] of given `len` length and `ct` `CellType`
    ///
    /// All cells will be filled with the `CellType`'s corresponding default value.
    fn with_defaults(len: usize, ct: CellType) -> Self;

    /// Create a buffer of size `len` with all values `value`.
    fn fill(len: usize, value: CellValue) -> Self;

    /// Fill a buffer of size `len` with values from a closure.
    ///
    /// First parameter of the closure is the current index.  
    fn fill_with<T, F>(len: usize, f: F) -> Self
    where
        T: CellEncoding,
        F: Fn(usize) -> T;

    /// Get the length of the buffer.
    fn len(&self) -> usize;

    /// Determine if the buffer has zero values in it.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the cell type of the encoded value.
    fn cell_type(&self) -> CellType;

    /// Get the [`CellValue`] at index `idx`.
    ///
    /// Panics of `idx` is outside of `[0, len())`.
    fn get(&self, index: usize) -> CellValue;

    /// Store `value` at position `idx`.
    fn put(&mut self, idx: usize, value: CellValue) -> error::Result<()>;

    /// Create a new [`CellBuffer`] whereby all [`CellValue`]s are converted to `cell_type`.
    ///
    /// Returns `Ok(CellBuffer)` if conversion is possible, and `Err(Error)` if
    /// contained values cannot fit in `cell_type` without clamping.
    fn convert(&self, cell_type: CellType) -> error::Result<Self>
    where
        Self: Sized;

    /// Compute the minimum and maximum values the buffer.
    fn min_max(&self) -> (CellValue, CellValue);

    /// Convert `self` into a `Vec<T>`.
    fn to_vec<T: CellEncoding>(self) -> error::Result<Vec<T>>;
}

/// Newtype wrapper for debug rendering utility.
pub(crate) struct Elided<'a, T>(&'a [T]);

impl<T: Debug> Debug for Elided<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        static MAX_LEN: usize = 10;
        fn render<T: Debug>(values: &[T], f: &mut Formatter<'_>) -> std::fmt::Result {
            match values.len() {
                len if len > MAX_LEN => {
                    render(&values[..5], f)?;
                    f.write_str(", ... ")?;
                    render(&values[len - 5..], f)?;
                }
                1 => {
                    f.write_fmt(format_args!("{:?}", values[0]))?;
                }
                len => {
                    for i in 0..(len - 1) {
                        render(&values[i..=i], f)?;
                        f.write_str(", ")?;
                    }
                    render(std::slice::from_ref(&values[len - 1]), f)?;
                }
            }
            Ok(())
        }

        render(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::Elided;

    #[test]
    fn elided() {
        let s = format!("{:?}", Elided(&vec![1; 3]));
        assert_eq!(s, "1, 1, 1");
        let s = format!("{:?}", Elided(&vec![0; 30]));
        assert_eq!(s, "0, 0, 0, 0, 0, ... 0, 0, 0, 0, 0");
    }
}
