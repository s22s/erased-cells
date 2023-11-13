/*
 * Copyright (c) 2023. Astraea, Inc. All rights reserved.
 */

use super::*;
use std::fmt::Debug;

/// Trait for marking Rust primitives as having a corresponding [`CellType`].
///
/// For example, [`f64`] is [`CellEncoding`] through [`CellType::Float64`],
/// but [`isize`] is not `CellEncoding`.
pub trait CellEncoding: Copy + Debug + Default {
    type Encoding: CellEncoding;
    /// Returns the [`CellType`] covering `Self`.
    fn cell_type() -> CellType;
    /// Converts `self` into a [`CellValue`].
    fn into_cell_value(self) -> CellValue;
    /// Convert dynamic type to static type when logically known.
    /// Returns `None` if given value isn't actually the exact same
    /// type as encoding.,
    fn cast<T: CellEncoding + Sized>(value: T) -> Option<Self> {
        if Self::cell_type() == T::cell_type() {
            Some(unsafe { std::mem::transmute_copy::<T, Self>(&value) })
        } else {
            None
        }
    }
}

/// Implements [`CellEncoding`] for each cell type.
macro_rules! encoding {
    ( $( ($ct:ident, $prim:ident) ),* ) => { $(
        impl CellEncoding for $prim {
            type Encoding = $prim;
            fn cell_type() -> CellType {
                CellType::$ct
            }
            fn into_cell_value(self) -> CellValue {
                CellValue::$ct(self)
            }
        } )*
    };
}

with_ct!(encoding);
