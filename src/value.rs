/*
 * Copyright (c) 2023. Astraea, Inc. All rights reserved.
 */

use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::with_ct;
use crate::{
    error::{Error, Result},
    CellEncoding, CellType,
};

/// CellValue enum constructor.
macro_rules! cv_enum {
    ( $(($id:ident, $p:ident)),*) => {
        /// Value variants for each [`CellType`]
        #[derive(Debug, Copy, Clone, Serialize, Deserialize)]
        pub enum CellValue { $($id($p)),* }
    }
}
with_ct!(cv_enum);

impl CellValue {
    pub fn new<T: CellEncoding + Sized>(value: T) -> Self {
        macro_rules! ctor {
            ( $( ($id:ident, $p:ident) ),*) => {
                match T::cell_type() {
                    $(CellType::$id => CellValue::$id($p::static_cast(value).unwrap()),)*
                }
            };
        }
        with_ct!(ctor)
    }

    /// Get the [`CellType`] encoding `self`.
    pub fn cell_type(&self) -> CellType {
        macro_rules! cv_ct {
            ($( ($id:ident, $_p:ident) ),*) => {
                match self {
                    $(CellValue::$id(_) => CellType::$id),*
                }
            };
        }
        with_ct!(cv_ct)
    }

    /// Get the [`CellValue`] contents as a `T`.
    ///
    /// Returns `Ok(T)` if the [`CellType`] of `T` is the same or wider than
    /// the encoded value, or `Err(Error)` if `T` is narrower.
    pub fn get<T: CellEncoding>(&self) -> Result<T> {
        let err = || Error::NarrowingError {
            src: self.cell_type(),
            dst: T::cell_type(),
        };
        let conv = self.convert(T::cell_type())?;
        macro_rules! conv {
             ($( ($id:ident, $_p:ident) ),*) => {
                match conv {
                    $(
                    CellValue::$id(v) => T::static_cast(v).ok_or_else(err),
                    )*
                }
            };
        }

        with_ct!(conv)
    }

    /// Convert `self` into a variant with [`CellType`] `cell_type` equal to or wider than
    /// its current `CellType`.
    ///
    /// Returns `Ok(CellValue)` if the [`CellType`] of `cell_type` is the same or wider than
    /// the encoded value, or `Err(Error)` if `T` is narrower.
    pub fn convert(&self, cell_type: CellType) -> Result<Self> {
        // TODO: Do something more like `GDALAdjustValueToDataType`
        let err = || Error::NarrowingError { src: self.cell_type(), dst: cell_type };
        if self.cell_type().union(cell_type) != cell_type {
            return Err(err());
        }

        if cell_type == self.cell_type() {
            return Ok(*self);
        }

        match cell_type {
            CellType::UInt8 => Ok(self.to_u8().ok_or_else(err)?.into_cell_value()),
            CellType::UInt16 => Ok(self.to_u16().ok_or_else(err)?.into_cell_value()),
            CellType::UInt32 => Ok(self.to_u32().ok_or_else(err)?.into_cell_value()),
            CellType::UInt64 => Ok(self.to_u64().ok_or_else(err)?.into_cell_value()),
            CellType::Int8 => Ok(self.to_i8().ok_or_else(err)?.into_cell_value()),
            CellType::Int16 => Ok(self.to_i16().ok_or_else(err)?.into_cell_value()),
            CellType::Int32 => Ok(self.to_i32().ok_or_else(err)?.into_cell_value()),
            CellType::Int64 => Ok(self.to_i64().ok_or_else(err)?.into_cell_value()),
            CellType::Float32 => Ok(self.to_f32().ok_or_else(err)?.into_cell_value()),
            CellType::Float64 => Ok(self.to_f64().ok_or_else(err)?.into_cell_value()),
        }
    }

    /// Determines the smallest cell type that can contain `self` and `other`, and then
    /// converts values to that cell type and returns a tuple of the converted values, i.e.
    /// `(convert(self), convert(other))`.
    pub fn unify(&self, other: &Self) -> (Self, Self) {
        let dest = self.cell_type().union(other.cell_type());
        // `unwrap` should be ok as it assumes `CellType::union` is correct.
        (self.convert(dest).unwrap(), other.convert(dest).unwrap())
    }
}

impl From<CellValue> for f64 {
    fn from(value: CellValue) -> Self {
        value.to_f64().expect("f64 conversion")
    }
}

impl<T: CellEncoding> From<T> for CellValue {
    fn from(value: T) -> Self {
        value.into_cell_value()
    }
}

impl ToPrimitive for CellValue {
    fn to_i64(&self) -> Option<i64> {
        macro_rules! conv {
            ($( ($id:ident, $_p:ident) ),*) => {
                match self {
                    $(
                    CellValue::$id(v) => v.to_i64(),
                    )*
                }
            }
        }
        with_ct!(conv)
    }

    fn to_u64(&self) -> Option<u64> {
        macro_rules! conv {
            ($( ($id:ident, $_p:ident) ),*) => {
                match self {
                    $(
                    CellValue::$id(v) => v.to_u64(),
                    )*
                }
            }
        }
        with_ct!(conv)
    }

    fn to_f64(&self) -> Option<f64> {
        macro_rules! conv {
            ($( ($id:ident, $_p:ident) ),*) => {
                match self {
                    $(
                    CellValue::$id(v) => v.to_f64(),
                    )*
                }
            }
        }
        with_ct!(conv)
    }
}

pub(crate) mod ops {
    use std::{
        cmp::Ordering,
        ops::{Add, Div, Mul, Neg, Sub},
    };

    use num_traits::ToPrimitive;

    use crate::{with_ct, CellValue};

    // NOTE: We currently take the position that any math ops will promote all integral primitives to f64 first
    macro_rules! cv_bin_op {
        ($trt:ident, $mth:ident, $op:tt) => {
            impl $trt for CellValue {
                type Output = CellValue;
                fn $mth(self, rhs: Self) -> Self::Output {
                    let (lhs, rhs) = self.unify(&rhs);
                    CellValue::new(<f64>::from(lhs) $op <f64>::from(rhs))
                }
            }
        }
    }

    cv_bin_op!(Add, add, +);
    cv_bin_op!(Sub, sub, -);
    cv_bin_op!(Mul, mul, *);
    cv_bin_op!(Div, div, /);

    impl Neg for CellValue {
        type Output = CellValue;
        fn neg(self) -> Self::Output {
            match self {
                CellValue::UInt8(v) => CellValue::new(-(v as i16)),
                CellValue::UInt16(v) => CellValue::new(-(v as i32)),
                CellValue::UInt32(v) => CellValue::new(-(v as i64)),
                CellValue::UInt64(v) => CellValue::new(-(v as f64)),
                CellValue::Int8(v) => CellValue::new(-v),
                CellValue::Int16(v) => CellValue::new(-v),
                CellValue::Int32(v) => CellValue::new(-v),
                CellValue::Int64(v) => CellValue::new(-v),
                CellValue::Float32(v) => CellValue::new(-v),
                CellValue::Float64(v) => CellValue::new(-v),
            }
        }
    }

    impl PartialOrd for CellValue {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            let (lhs, rhs) = self.unify(other);
            macro_rules! ord {
                ($( ($id:ident, $_p:ident) ),*) => {
                    match (lhs, rhs) {
                        $((CellValue::$id(l), CellValue::$id(r)) => l.partial_cmp(&r),)*
                        _ => unreachable!("{self:?} <> {other:?}"),
                    }
                };
            }
            with_ct!(ord)
        }
    }

    impl Ord for CellValue {
        fn cmp(&self, other: &Self) -> Ordering {
            self.partial_cmp(other)
                .unwrap_or_else(|| self.to_f64().unwrap().total_cmp(&other.to_f64().unwrap()))
        }
    }

    impl PartialEq<Self> for CellValue {
        fn eq(&self, other: &Self) -> bool {
            let (lhs, rhs) = self.unify(other);
            macro_rules! cmp {
                ($( ($id:ident, $_p:ident) ),*) => {
                    match (lhs, rhs) {
                        $((CellValue::$id(l), CellValue::$id(r)) => l.eq(&r),)*
                        _ => unreachable!("{self:?} <> {other:?}"),
                    }
                };
            }
            with_ct!(cmp)
        }
    }

    impl Eq for CellValue {}
}

#[cfg(test)]
mod tests {
    use crate::with_ct;
    use crate::{CellType, CellValue};

    #[test]
    fn cell_type() {
        // Confirm CellValue::cell_type is correct.
        macro_rules! test {
            ($( ($id:ident, $p:ident) ),*) => {
                $(assert_eq!(CellValue::$id($p::default()).cell_type(), CellType::$id);)*
            };
        }
        with_ct!(test);
    }

    #[test]
    fn get() {
        macro_rules! test {
            ($( ($id:ident, $p:ident) ),*) => {
                $({
                    let v = $p::default();
                    let cv = CellValue::new(v);
                    let r = cv.get::<$p>();
                    assert!(r.is_ok());
                    assert_eq!(r.unwrap(), v);
                    let r2 = cv.get::<f64>();
                    assert!(r2.is_ok(), "{:?}", cv);
                    assert_eq!(r2.unwrap(), v as f64)
                })*
            }
        }
        with_ct!(test);
    }

    #[test]
    fn convert() {
        assert!(matches!(
            CellValue::UInt8(43).convert(CellType::Int16),
            Ok(CellValue::Int16(43))
        ));
        assert!(CellValue::Float32(3.14).convert(CellType::Int64).is_err())
    }

    #[test]
    #[allow(illegal_floating_point_literal_pattern)]
    fn unary() {
        assert!(matches!(-CellValue::UInt8(1), CellValue::Int16(-1)));
        assert!(matches!(-CellValue::UInt16(1), CellValue::Int32(-1)));
        assert!(matches!(-CellValue::Int8(1), CellValue::Int8(-1)));
        assert!(matches!(-CellValue::Int16(1), CellValue::Int16(-1)));
        assert!(matches!(-CellValue::Float64(1.0), CellValue::Float64(-1.0)));
        assert!(matches!(-CellValue::Float32(1.0), CellValue::Float32(-1.0)));
    }

    #[test]
    fn binops() {
        let l = CellValue::UInt8(1);
        let r = CellValue::UInt8(2);
        assert_eq!(l + r, CellValue::Float64(3.));
        assert_eq!(l - r, CellValue::Float64(-1.));
        assert_eq!(r - l, CellValue::Float64(1.));
        assert_eq!(l * r, CellValue::Float64(2.));
        assert_eq!(r * l, CellValue::Float64(2.));
        assert_eq!(l / r, CellValue::Float64(0.5));
        assert_eq!(r / l, CellValue::Float64(2.));

        let l = CellValue::UInt16(1);
        let r = CellValue::UInt16(2);
        assert_eq!(l + r, CellValue::Float64(3.));
        assert_eq!(l - r, CellValue::Float64(-1.));
        assert_eq!(r - l, CellValue::Float64(1.));
        assert_eq!(l * r, CellValue::Float64(2.));
        assert_eq!(r * l, CellValue::Float64(2.));
        assert_eq!(l / r, CellValue::Float64(0.5));
        assert_eq!(r / l, CellValue::Float64(2.));

        let l = CellValue::Float32(1.);
        let r = CellValue::Float32(2.);
        assert_eq!(l + r, CellValue::Float32(3.));
        assert_eq!(l - r, CellValue::Float32(-1.));
        assert_eq!(r - l, CellValue::Float32(1.));
        assert_eq!(l * r, CellValue::Float32(2.));
        assert_eq!(r * l, CellValue::Float32(2.));
        assert_eq!(l / r, CellValue::Float32(0.5));
        assert_eq!(r / l, CellValue::Float32(2.));

        let l = CellValue::Float64(1.);
        let r = CellValue::Float64(2.);
        assert_eq!(l + r, CellValue::Float64(3.));
        assert_eq!(l - r, CellValue::Float64(-1.));
        assert_eq!(r - l, CellValue::Float64(1.));
        assert_eq!(l * r, CellValue::Float64(2.));
        assert_eq!(r * l, CellValue::Float64(2.));
        assert_eq!(l / r, CellValue::Float64(0.5));
        assert_eq!(r / l, CellValue::Float64(2.));
    }
}
