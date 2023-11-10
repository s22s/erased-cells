/*
 * Copyright (c) 2023. Astraea, Inc. All rights reserved.
 */

use crate::error::{Error, Result};
use crate::{CellEncoding, CellType, CellValue, HasCellType};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum CellBuffer {
    UInt8(Vec<u8>),
    UInt16(Vec<u16>),
    // Int16(Vec<i16>),
    // Int32(Vec<i32>),
    Float32(Vec<f32>),
    Float64(Vec<f64>),
}

impl CellBuffer {
    pub fn new<T: CellEncoding>(data: Vec<T>) -> Self {
        data.into()
    }
    pub fn empty(len: usize, ct: CellType) -> Self {
        match ct {
            CellType::UInt8 => Self::new(vec![u8::default(); len]),
            CellType::UInt16 => Self::new(vec![u16::default(); len]),
            CellType::Float32 => Self::new(vec![f32::default(); len]),
            CellType::Float64 => Self::new(vec![f64::default(); len]),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            CellBuffer::UInt8(v) => v.len(),
            CellBuffer::UInt16(v) => v.len(),
            CellBuffer::Float32(v) => v.len(),
            CellBuffer::Float64(v) => v.len(),
        }
    }

    pub fn cell_type(&self) -> CellType {
        match self {
            CellBuffer::UInt8(_) => CellType::UInt8,
            CellBuffer::UInt16(_) => CellType::UInt16,
            CellBuffer::Float32(_) => CellType::Float32,
            CellBuffer::Float64(_) => CellType::Float64,
        }
    }

    /// Panics of `idx` is outside of `[0, len())`.
    pub fn get(&self, idx: usize) -> CellValue {
        match self {
            CellBuffer::UInt8(b) => CellValue::UInt8(b[idx]),
            CellBuffer::UInt16(b) => CellValue::UInt16(b[idx]),
            CellBuffer::Float32(b) => CellValue::Float32(b[idx]),
            CellBuffer::Float64(b) => CellValue::Float64(b[idx]),
        }
    }

    pub fn put(&mut self, idx: usize, value: CellValue) -> Result<()> {
        let value = value.convert(self.cell_type())?;
        match self {
            CellBuffer::UInt8(b) => match value {
                CellValue::UInt8(v) => b[idx] = v,
                _ => unreachable!(),
            },
            CellBuffer::UInt16(b) => match value {
                CellValue::UInt16(v) => b[idx] = v,
                _ => unreachable!(),
            },
            CellBuffer::Float32(b) => match value {
                CellValue::Float32(v) => b[idx] = v,
                _ => unreachable!(),
            },
            CellBuffer::Float64(b) => match value {
                CellValue::Float64(v) => b[idx] = v,
                _ => unreachable!(),
            },
        }

        Ok(())
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = CellValue> + '_> {
        // TODO: get rid of clone!
        match self {
            Self::UInt8(v) => Box::new(v.iter().map(|v| (*v).into())),
            Self::UInt16(v) => Box::new(v.iter().map(|v| (*v).into())),
            Self::Float32(v) => Box::new(v.iter().map(|v| (*v).into())),
            Self::Float64(v) => Box::new(v.iter().map(|v| (*v).into())),
        }
    }

    pub fn convert(&self, cell_type: CellType) -> Result<Self> {
        let err = || Err(Error::NarrowingError { src: self.cell_type(), dst: cell_type });
        match self {
            Self::UInt8(b) => match cell_type {
                CellType::UInt8 => Ok(self.clone()),
                CellType::UInt16 => Ok(Self::UInt16(b.into_iter().map(|&x| x as u16).collect())),
                CellType::Float32 => Ok(Self::Float32(b.into_iter().map(|&x| x as f32).collect())),
                CellType::Float64 => Ok(Self::Float64(b.into_iter().map(|&x| x as f64).collect())),
            },
            Self::UInt16(b) => match cell_type {
                CellType::UInt8 => err(),
                CellType::UInt16 => Ok(self.clone()),
                CellType::Float32 => Ok(Self::Float32(b.into_iter().map(|&x| x as f32).collect())),
                CellType::Float64 => Ok(Self::Float64(b.into_iter().map(|&x| x as f64).collect())),
            },
            Self::Float32(b) => match cell_type {
                CellType::UInt8 => err(),
                CellType::UInt16 => err(),
                CellType::Float32 => Ok(self.clone()),
                CellType::Float64 => Ok(Self::Float64(b.into_iter().map(|&x| x as f64).collect())),
            },
            Self::Float64(_) => match cell_type {
                CellType::UInt8 => err(),
                CellType::UInt16 => err(),
                CellType::Float32 => err(),
                CellType::Float64 => Ok(self.clone()),
            },
        }
    }

    pub fn minmax(&self) -> (CellValue, CellValue) {
        let init = (self.cell_type().max(), self.cell_type().min());
        self.iter()
            .fold(init, |(amin, amax), v| (amin.min(v), amax.max(v)))
    }

    pub fn to_vec<T: CellEncoding>(&self) -> Result<Vec<T>> {
        let r = self.convert(T::cell_type())?;

        match r {
            CellBuffer::UInt8(b) => Ok(danger::cast(b)),
            CellBuffer::UInt16(b) => Ok(danger::cast(b)),
            CellBuffer::Float32(b) => Ok(danger::cast(b)),
            CellBuffer::Float64(b) => Ok(danger::cast(b)),
        }
    }
}

impl HasCellType for CellBuffer {
    fn cell_type(&self) -> CellType {
        self.cell_type()
    }
}

impl From<&CellBuffer> for CellType {
    fn from(value: &CellBuffer) -> Self {
        value.cell_type()
    }
}

impl<T: CellEncoding> From<Vec<T>> for CellBuffer {
    fn from(values: Vec<T>) -> Self {
        match T::cell_type() {
            CellType::UInt8 => Self::UInt8(danger::cast(values)),
            CellType::UInt16 => Self::UInt16(danger::cast(values)),
            CellType::Float32 => Self::Float32(danger::cast(values)),
            CellType::Float64 => Self::Float64(danger::cast(values)),
        }
    }
}

impl<T: CellEncoding> From<&[T]> for CellBuffer {
    fn from(values: &[T]) -> Self {
        match T::cell_type() {
            CellType::UInt8 => Self::UInt8(danger::cast(values.to_vec())),
            CellType::UInt16 => Self::UInt16(danger::cast(values.to_vec())),
            CellType::Float32 => Self::Float32(danger::cast(values.to_vec())),
            CellType::Float64 => Self::Float64(danger::cast(values.to_vec())),
        }
    }
}

impl Debug for CellBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let basename = self.cell_type().to_string();

        fn render_values<T: ToString>(values: &Vec<T>) -> String {
            if values.len() > 10 {
                let values = values.as_slice();
                let mut front = values[values.len() - 5..]
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                front.push(String::from("..."));
                values[..5]
                    .iter()
                    .map(ToString::to_string)
                    .for_each(|s| front.push(s));
                front.join(", ")
            } else {
                values
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        }

        let values = match self {
            CellBuffer::UInt8(b) => render_values(b),
            CellBuffer::UInt16(b) => render_values(b),
            CellBuffer::Float32(b) => render_values(b),
            CellBuffer::Float64(b) => render_values(b),
        };
        f.debug_struct(&format!("{basename}CellBuffer"))
            .field("values", &values)
            .finish()
    }
}

impl FromIterator<f64> for CellBuffer {
    fn from_iter<T: IntoIterator<Item = f64>>(iter: T) -> Self {
        Self::Float64(iter.into_iter().collect())
    }
}

impl FromIterator<CellValue> for CellBuffer {
    fn from_iter<T: IntoIterator<Item = CellValue>>(iterable: T) -> Self {
        let values = iterable.into_iter().collect::<Vec<CellValue>>();

        match values.as_slice() {
            [] => CellBuffer::UInt8(Vec::new()),
            [x, ..] => {
                let ct: CellType = x.cell_type();
                match ct {
                    CellType::UInt8 => {
                        CellBuffer::UInt8(values.iter().map(|v| v.get().unwrap()).collect())
                    }
                    CellType::UInt16 => {
                        CellBuffer::UInt16(values.iter().map(|v| v.get().unwrap()).collect())
                    }
                    CellType::Float32 => {
                        CellBuffer::Float32(values.iter().map(|v| v.get().unwrap()).collect())
                    }
                    CellType::Float64 => {
                        CellBuffer::Float64(values.iter().map(|v| v.get().unwrap()).collect())
                    }
                }
            }
        }
    }
}

pub(crate) mod ops {
    use crate::CellBuffer;
    use std::ops::{Add, Div, Mul, Neg, Sub};

    macro_rules! cb_bin_op {
        ($trt:ident, $mth:ident, $op:tt) => {
            impl $trt for &CellBuffer {
                type Output = CellBuffer;
                fn $mth(self, rhs: Self) -> Self::Output {
                    self.iter().zip(rhs.iter()).map(|(l, r)| l $op r).collect()
                }
            }
        }
    }
    cb_bin_op!(Add, add, +);
    cb_bin_op!(Sub, sub, -);
    cb_bin_op!(Mul, mul, *);
    cb_bin_op!(Div, div, /);

    impl Neg for &CellBuffer {
        type Output = CellBuffer;
        fn neg(self) -> Self::Output {
            self.iter().map(|v| -v).collect()
        }
    }
}

mod danger {
    use crate::CellEncoding;

    #[inline]
    pub(crate) fn cast<T: CellEncoding, P: CellEncoding>(buffer: Vec<T>) -> Vec<P> {
        assert_eq!(T::cell_type(), P::cell_type());
        // As suggested in https://doc.rust-lang.org/core/intrinsics/fn.transmute.html
        unsafe {
            let mut v = std::mem::ManuallyDrop::new(buffer);
            Vec::from_raw_parts(v.as_mut_ptr() as *mut P, v.len(), v.capacity())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{CellBuffer, CellValue};

    #[test]
    fn minmax() {
        let buf = CellBuffer::new(vec![-1.0, 3.0, 2000.0, -5555.5]);
        let (min, max) = buf.minmax();
        assert_eq!(min, CellValue::Float64(-5555.5));
        assert_eq!(max, CellValue::Float64(2000.0));

        let buf = CellBuffer::new(vec![1u8, 3u8, 200u8, 0u8]);
        let (min, max) = buf.minmax();
        assert_eq!(min, CellValue::UInt8(0));
        assert_eq!(max, CellValue::UInt8(200));
    }
}
