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
//! When the `masked` feature is enabled (the default) three additional constructs are available:
//!
//! * [`Mask`]: A bit array used to store the associated validity of a [`CellValue`] within a [`MaskedCellBuffer`].
//! * [`MaskedCellBuffer`]: The combination of  [`CellBuffer`] and [`Mask`].
//! * [`NoData`]: Specification of a sentinel value for invalid data, used in converting
//! between "`Vec<T: CellEncoding>`" and [`MaskedCellBuffer`].
//!
//! # Examples
//!
//! Usage examples:
//!
//! * [`CellBuffer` example](crate::CellBuffer#example)
//! * [`MaskedCellBuffer` example](crate::MaskedCellBuffer#example)
//!
//! # Feature Flags
//!
//! The following feature flags are available.
//!
//! | Name     | Description                                    | Default |
//! |:--------:|------------------------------------------------|:-------:|
//! | `masked` | Enable the `MaskedCellBuffer` API              | `true`  |
//! | `serde`  | Derive `serde` traits for core types           | `true`  |
//! | `gdal`   | Enable `CellBuffer`s in `georust/gdal` API     | `false` |
//!

mod buffer;
mod ctype;
mod encoding;
pub mod error;
#[cfg(feature = "gdal")]
mod gdal;
#[cfg(feature = "masked")]
mod masked;
mod value;

pub use buffer::*;
pub use ctype::*;
pub use encoding::*;
#[cfg(feature = "masked")]
pub use masked::*;
use std::fmt::{Debug, Formatter};
pub use value::ops::*;
pub use value::*;

#[cfg(not(feature = "gdal"))]
#[doc=include_str!("with_ct.md")]
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
#[cfg(feature = "gdal")]
#[doc=include_str!("with_ct.md")]
#[macro_export]
macro_rules! with_ct {
    ($callback:ident) => {
        $callback! {
            (UInt8, u8),
            (UInt16, u16),
            (UInt32, u32),
            (Int16, i16),
            (Int32, i32),
            (Float32, f32),
            (Float64, f64)
        }
    };
}

/// Operations common to buffers of [`CellValue`]s.
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
    fn fill_via<T, F>(len: usize, f: F) -> Self
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
    /// # Panics
    /// Will panic if `index` >= `self.len()`.
    fn get(&self, index: usize) -> CellValue;

    /// Store `value` at position `idx`.
    ///
    /// Returns `Err(NarrowingError)` if `value.cell_type() != self.cell_type()`
    /// and overflow could occur.
    ///
    /// # Panics
    /// Will panic if `index` >= `self.len()`.
    fn put(&mut self, index: usize, value: CellValue) -> error::Result<()>;

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

        render(self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::Elided;

    #[test]
    fn elided() {
        let s = format!("{:?}", Elided(&[1; 3]));
        assert_eq!(s, "1, 1, 1");
        let s = format!("{:?}", Elided(&[0; 30]));
        assert_eq!(s, "0, 0, 0, 0, 0, ... 0, 0, 0, 0, 0");
    }
}
