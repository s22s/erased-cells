/*
 * Copyright (c) 2023. Astraea, Inc. All rights reserved.
 */

use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    CellEncoding, CellType,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum CellValue {
    UInt8(u8),
    UInt16(u16),
    // Int16(i16),
    // Int32(i32),
    Float32(f32),
    Float64(f64),
}

impl CellValue {
    pub fn cell_type(&self) -> CellType {
        match self {
            CellValue::UInt8(_) => CellType::UInt8,
            CellValue::UInt16(_) => CellType::UInt16,
            CellValue::Float32(_) => CellType::Float32,
            CellValue::Float64(_) => CellType::Float64,
        }
    }
    pub fn get<T: CellEncoding>(&self) -> Result<T> {
        let err = || Error::NarrowingError {
            src: self.cell_type(),
            dst: T::cell_type(),
        };
        match T::cell_type() {
            CellType::UInt8 => match self {
                CellValue::UInt8(v) => T::cast(*v).ok_or_else(err),
                _ => Err(err()),
            },
            CellType::UInt16 => match self {
                CellValue::UInt16(v) => T::cast(*v).ok_or_else(err),
                _ => Err(err()),
            },
            CellType::Float32 => match self {
                CellValue::Float32(v) => T::cast(*v).ok_or_else(err),
                _ => Err(err()),
            },
            CellType::Float64 => match self {
                CellValue::Float64(v) => T::cast(*v).ok_or_else(err),
                _ => Err(err()),
            },
        }
    }

    pub fn convert(&self, cell_type: CellType) -> Result<Self> {
        // TODO: Do something more like `GDALAdjustValueToDataType`
        let err = || Err(Error::NarrowingError { src: self.cell_type(), dst: cell_type });
        match self {
            Self::UInt8(v) => match cell_type {
                CellType::UInt8 => Ok(self.clone()),
                CellType::UInt16 => Ok(Self::UInt16(*v as u16)),
                CellType::Float32 => Ok(Self::Float32(*v as f32)),
                CellType::Float64 => Ok(Self::Float64(*v as f64)),
            },
            Self::UInt16(v) => match cell_type {
                CellType::UInt8 => err(),
                CellType::UInt16 => Ok(self.clone()),
                CellType::Float32 => Ok(Self::Float32(*v as f32)),
                CellType::Float64 => Ok(Self::Float64(*v as f64)),
            },
            Self::Float32(v) => match cell_type {
                CellType::UInt8 => err(),
                CellType::UInt16 => err(),
                CellType::Float32 => Ok(self.clone()),
                CellType::Float64 => Ok(Self::Float64(*v as f64)),
            },
            Self::Float64(_) => match cell_type {
                CellType::UInt8 => err(),
                CellType::UInt16 => err(),
                CellType::Float32 => err(),
                CellType::Float64 => Ok(self.clone()),
            },
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
        value.to_f64().unwrap_or(f64::NAN)
    }
}

impl<T: CellEncoding> From<T> for CellValue {
    fn from(value: T) -> Self {
        value.into_cell_value()
    }
}

impl ToPrimitive for CellValue {
    fn to_i64(&self) -> Option<i64> {
        match self {
            CellValue::UInt8(v) => v.to_i64(),
            CellValue::UInt16(v) => v.to_i64(),
            CellValue::Float32(v) => v.to_i64(),
            CellValue::Float64(v) => v.to_i64(),
        }
    }

    fn to_u64(&self) -> Option<u64> {
        match self {
            CellValue::UInt8(v) => v.to_u64(),
            CellValue::UInt16(v) => v.to_u64(),
            CellValue::Float32(v) => v.to_u64(),
            CellValue::Float64(v) => v.to_u64(),
        }
    }

    fn to_f64(&self) -> Option<f64> {
        match self {
            CellValue::UInt8(v) => v.to_f64(),
            CellValue::UInt16(v) => v.to_f64(),
            CellValue::Float32(v) => v.to_f64(),
            CellValue::Float64(v) => Some(*v),
        }
    }
}

pub(crate) mod ops {
    use std::{
        cmp::Ordering,
        ops::{Add, Div, Mul, Neg, Sub},
    };

    use num_traits::ToPrimitive;

    use crate::CellValue;

    // NOTE: We currently take the position that any math ops will promote all integral primitives to f64 first
    macro_rules! cv_bin_op {
        ($trt:ident, $mth:ident, $op:tt) => {
            impl $trt for CellValue {
                type Output = CellValue;
                fn $mth(self, rhs: Self) -> Self::Output {
                    match self.unify(&rhs) {
                        (CellValue::UInt8(l), CellValue::UInt8(r)) => CellValue::Float64((l as f64) $op (r as f64)),
                        (CellValue::UInt16(l), CellValue::UInt16(r)) => CellValue::Float64((l as f64) $op (r as f64)),
                        (CellValue::Float32(l), CellValue::Float32(r)) => CellValue::Float32(l $op r),
                        (CellValue::Float64(l), CellValue::Float64(r)) => CellValue::Float64(l $op r),
                        o => unimplemented!("{o:?}"),
                    }
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
                // TODO: once we have signed types we should widen to the closer type
                CellValue::UInt8(v) => CellValue::Float64(-(v as f64)),
                CellValue::UInt16(v) => CellValue::Float64(-(v as f64)),
                CellValue::Float32(v) => CellValue::Float32(-v),
                CellValue::Float64(v) => CellValue::Float64(-v),
            }
        }
    }

    impl PartialOrd for CellValue {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            let (lhs, rhs) = self.unify(other);

            match (lhs, rhs) {
                (CellValue::UInt8(l), CellValue::UInt8(r)) => l.partial_cmp(&r),
                (CellValue::UInt16(l), CellValue::UInt16(r)) => l.partial_cmp(&r),
                (CellValue::Float32(l), CellValue::Float32(r)) => l.partial_cmp(&r),
                (CellValue::Float64(l), CellValue::Float64(r)) => l.partial_cmp(&r),
                _ => unreachable!(),
            }
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

            match (lhs, rhs) {
                (CellValue::UInt8(l), CellValue::UInt8(r)) => l.eq(&r),
                (CellValue::UInt16(l), CellValue::UInt16(r)) => l.eq(&r),
                (CellValue::Float32(l), CellValue::Float32(r)) => l.eq(&r),
                (CellValue::Float64(l), CellValue::Float64(r)) => l.eq(&r),
                _ => unreachable!(),
            }
        }
    }

    impl Eq for CellValue {}
}

#[cfg(test)]
mod tests {
    use crate::CellValue;

    #[test]
    fn unary() {
        let l = CellValue::UInt8(1);
        assert_eq!(-l, CellValue::Float64(-1.0));
        let l = CellValue::UInt16(1);
        assert_eq!(-l, CellValue::Float64(-1.0));
        let l = CellValue::Float64(1.0);
        assert_eq!(-l, CellValue::Float64(-1.0));
        let l = CellValue::Float32(1.0);
        assert_eq!(-l, CellValue::Float32(-1.0));
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
