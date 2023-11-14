/*
 * Copyright (c) 2023. Astraea, Inc. All rights reserved.
 */

use crate::error::{Error, Result};
use crate::{with_ct, CellEncoding, CellType, CellValue};
use std::fmt::{Debug, Formatter};

/// CellBuffer enum constructor.
macro_rules! cv_enum {
    ( $(($id:ident, $p:ident)),*) => {
        /// Buffer variants for each [`CellType`]
        #[derive(Clone)]
        pub enum CellBuffer { $($id(Vec<$p>)),* }
    }
}
with_ct!(cv_enum);

impl CellBuffer {
    pub fn new<T: CellEncoding>(data: Vec<T>) -> Self {
        data.into()
    }

    pub fn defaults(len: usize, ct: CellType) -> Self {
        macro_rules! empty {
            ( $(($id:ident, $p:ident)),*) => {
                match ct {
                    $(CellType::$id => Self::new(vec![$p::default(); len]),)*
                }
            };
        }
        with_ct!(empty)
    }

    pub fn fill(value: CellValue, len: usize) -> Self {
        macro_rules! empty {
            ( $(($id:ident, $p:ident)),*) => {
                match value.cell_type() {
                    $(CellType::$id => Self::new::<$p>(vec![value.get().unwrap(); len]),)*
                }
            };
        }
        with_ct!(empty)
    }

    pub fn len(&self) -> usize {
        macro_rules! len {
            ( $(($id:ident, $_p:ident)),*) => {
                match self {
                    $(CellBuffer::$id(v) => v.len(),)*
                }
            };
        }
        with_ct!(len)
    }

    pub fn cell_type(&self) -> CellType {
        macro_rules! ct {
            ( $(($id:ident, $_p:ident)),*) => {
                match self {
                    $(CellBuffer::$id(_) => CellType::$id,)*
                }
            };
        }
        with_ct!(ct)
    }

    /// Panics of `idx` is outside of `[0, len())`.
    pub fn get(&self, idx: usize) -> CellValue {
        macro_rules! get {
            ( $(($id:ident, $_p:ident)),*) => {
                match self {
                    $(CellBuffer::$id(b) => CellValue::$id(b[idx]),)*
                }
            };
        }
        with_ct!(get)
    }

    pub fn put(&mut self, idx: usize, value: CellValue) -> Result<()> {
        let value = value.convert(self.cell_type())?;
        macro_rules! put {
            ( $(($id:ident, $_p:ident)),*) => {
                match (self, value) {
                    $((CellBuffer::$id(b), CellValue::$id(v)) => b[idx] = v,)*
                    _ => unreachable!(),
                }
            }
        }
        with_ct!(put);
        Ok(())
    }

    pub fn convert(&self, cell_type: CellType) -> Result<Self> {
        let err = || Error::NarrowingError { src: self.cell_type(), dst: cell_type };

        if !self.cell_type().can_fit_into(cell_type) {
            return Err(err());
        }

        if cell_type == self.cell_type() {
            return Ok(self.clone());
        }

        let r: CellBuffer = self
            .into_iter()
            .map(|v| v.convert(cell_type).unwrap())
            .collect();

        Ok(r)
    }

    pub fn minmax(&self) -> (CellValue, CellValue) {
        let init = (self.cell_type().max(), self.cell_type().min());
        self.into_iter()
            .fold(init, |(amin, amax), v| (amin.min(v), amax.max(v)))
    }

    pub fn to_vec<T: CellEncoding>(&self) -> Result<Vec<T>> {
        let r = self.convert(T::cell_type())?;
        macro_rules! to_vec {
            ( $(($id:ident, $_p:ident)),*) => {
                match r {
                    $(CellBuffer::$id(b) => Ok(danger::cast(b)),)*
                }
            }
        }
        with_ct!(to_vec)
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
        macro_rules! render {
            ( $(($id:ident, $_p:ident)),*) => {
                match self {
                    $(CellBuffer::$id(b) => render_values(b),)*
                }
            }
        }

        let values = with_ct!(render);
        f.debug_struct(&format!("{basename}CellBuffer"))
            .field("values", &values)
            .finish()
    }
}

impl<C: CellEncoding> FromIterator<C> for CellBuffer {
    fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl FromIterator<CellValue> for CellBuffer {
    fn from_iter<T: IntoIterator<Item = CellValue>>(iterable: T) -> Self {
        // TODO: is there a way to avoid this collect?
        let values = iterable.into_iter().collect::<Vec<CellValue>>();
        match values.as_slice() {
            [] => CellBuffer::UInt8(Vec::new()),
            [x, ..] => {
                let ct: CellType = x.cell_type();
                macro_rules! conv {
                    ( $(($id:ident, $_p:ident)),*) => {
                        match ct {
                            $(CellType::$id => {
                                CellBuffer::$id(values.iter().map(|v| v.get().unwrap()).collect())
                            })*
                        }
                    }
                }
                with_ct!(conv)
            }
        }
    }
}

impl<T: CellEncoding> From<Vec<T>> for CellBuffer {
    fn from(values: Vec<T>) -> Self {
        macro_rules! from {
            ( $(($id:ident, $_p:ident)),*) => {
                match T::cell_type() {
                    $(CellType::$id => Self::$id(danger::cast(values)),)*
                }
            }
        }
        with_ct!(from)
    }
}

impl<T: CellEncoding> From<&[T]> for CellBuffer {
    fn from(values: &[T]) -> Self {
        macro_rules! from {
            ( $(($id:ident, $_p:ident)),*) => {
                match T::cell_type() {
                    $(CellType::$id => Self::$id(danger::cast(values.to_vec())),)*
                }
            }
        }
        with_ct!(from)
    }
}

impl<'buf> IntoIterator for &'buf CellBuffer {
    type Item = CellValue;
    type IntoIter = CellBufferIterator<'buf>;
    fn into_iter(self) -> Self::IntoIter {
        CellBufferIterator { buf: self, idx: 0, len: self.len() }
    }
}

pub struct CellBufferIterator<'buf> {
    buf: &'buf CellBuffer,
    idx: usize,
    len: usize,
}

impl Iterator for CellBufferIterator<'_> {
    type Item = CellValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            None
        } else {
            let r = self.buf.get(self.idx);
            self.idx += 1;
            Some(r)
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
                    self.into_iter().zip(rhs.into_iter()).map(|(l, r)| l $op r).collect()
                }
            }
            impl $trt for CellBuffer {
                type Output = CellBuffer;
                fn $mth(self, rhs: Self) -> Self::Output {
                    $trt::$mth(&self, &rhs)
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
            self.into_iter().into_iter().map(|v| -v).collect()
        }
    }
    impl Neg for CellBuffer {
        type Output = CellBuffer;
        fn neg(self) -> Self::Output {
            Neg::neg(&self)
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
    use crate::{with_ct, CellBuffer, CellType, CellValue};

    fn bigger(start: CellType) -> impl Iterator<Item = CellType> {
        CellType::iter().filter(move |ct| start.can_fit_into(*ct))
    }

    #[test]
    fn defaults() {
        macro_rules! test {
            ($( ($id:ident, $p:ident) ),*) => {
                $({
                    let cv = CellBuffer::defaults(3, CellType::$id);
                    assert_eq!(cv.len(), 3);
                    assert_eq!(cv.get(0), CellValue::new(<$p>::default()));
                })*};
        }
        with_ct!(test);
    }

    #[test]
    fn put_get() {
        use num_traits::One;
        macro_rules! test {
            ($( ($id:ident, $p:ident) ),*) => {
                $({
                    let mut cv = CellBuffer::fill(<$p>::default().into(), 3);
                    let one = CellValue::new(<$p>::one());
                    cv.put(1, one).expect("Put one");
                    assert_eq!(cv.get(1), one.convert(CellType::$id).unwrap());
                })*};
        }
        with_ct!(test);
    }

    #[test]
    fn to_vec() {
        macro_rules! test {
            ($( ($id:ident, $p:ident) ),*) => {
                $({
                    let v = vec![<$p>::default(); 3];
                    let buf = CellBuffer::new(v.clone());
                    let r = buf.to_vec::<$p>().unwrap();
                    assert_eq!(r, v);
                })*
            };
        }
        with_ct!(test);
    }

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

    #[test]
    fn from_others() {
        let v = vec![
            CellValue::UInt16(3),
            CellValue::UInt16(4),
            CellValue::UInt16(5),
        ];

        let b: CellBuffer = v.into_iter().collect();
        assert_eq!(b.cell_type(), CellType::UInt16);
        assert_eq!(b.len(), 3);
        assert_eq!(b.get(2), CellValue::UInt16(5));

        let v = vec![33.3f32, 44.4, 55.5];
        let b: CellBuffer = v.clone().into_iter().collect();
        assert_eq!(b.cell_type(), CellType::Float32);
        assert_eq!(b.len(), 3);
        assert_eq!(b.get(2), CellValue::Float32(55.5));

        let b: CellBuffer = v.clone().into();
        assert_eq!(b.cell_type(), CellType::Float32);
        assert_eq!(b.len(), 3);
        assert_eq!(b.get(2), CellValue::Float32(55.5));

        let b: CellBuffer = v.clone().as_slice().into();
        assert_eq!(b.cell_type(), CellType::Float32);
        assert_eq!(b.len(), 3);
        assert_eq!(b.get(2), CellValue::Float32(55.5));
    }

    #[test]
    fn debug() {
        let b = CellBuffer::fill(37.into(), 5);
        assert!(format!("{b:?}").starts_with("Int32CellBuffer"));
        let b = CellBuffer::fill(37.into(), 15);
        assert!(format!("{b:?}").contains("..."));
    }

    #[test]
    fn convert() {
        for ct in CellType::iter() {
            let buf = CellBuffer::defaults(3, ct);
            for target in bigger(ct) {
                let r = buf.convert(target);
                assert!(r.is_ok(), "{ct} vs {target}");
                let r = r.unwrap();

                assert_eq!(r.cell_type(), target);
            }
        }
    }

    #[test]
    fn unary() {
        use num_traits::One;
        macro_rules! test {
            ($( ($id:ident, $p:ident) ),*) => {$({
                let one: CellValue = <$p>::one().into();
                let buf = -CellBuffer::fill(one, 3);
                assert_eq!(buf.get(0), -one);
            })*};
        }

        with_ct!(test);
    }

    #[test]
    fn binary() {
        for lhs_ct in CellType::iter() {
            let lhs_val = lhs_ct.one();
            let lhs = CellBuffer::fill(lhs_val, 3);
            for rhs_ct in CellType::iter() {
                let rhs_val = rhs_ct.one() + rhs_ct.one();
                let rhs = CellBuffer::fill(rhs_val, 3);
                assert_eq!((&lhs + &rhs).get(1), lhs_val + rhs_val);
                assert_eq!((&rhs + &lhs).get(1), rhs_val + lhs_val);
                assert_eq!((&lhs - &rhs).get(1), lhs_val - rhs_val);
                assert_eq!((&rhs - &lhs).get(1), rhs_val - lhs_val);
                assert_eq!((&lhs * &rhs).get(1), lhs_val * rhs_val);
                assert_eq!((&rhs * &lhs).get(1), rhs_val * lhs_val);
                assert_eq!((&lhs / &rhs).get(1), lhs_val / rhs_val);
                assert_eq!((&rhs / &lhs).get(1), rhs_val / lhs_val);
            }
        }
    }
}
