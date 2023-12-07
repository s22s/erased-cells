use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

pub use self::ops::*;
use crate::{BufferOps, CellBuffer, CellEncoding, CellType, CellValue, Mask, NoData};

#[derive(Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MaskedCellBuffer(CellBuffer, Mask);

impl MaskedCellBuffer {
    /// Create a new combined [`CellBuffer`] and [`Mask`].
    ///
    /// Panics if `buffer` and `mask` are not the same length.
    pub fn new(buffer: CellBuffer, mask: Mask) -> Self {
        assert_eq!(
            buffer.len(),
            mask.len(),
            "Mask and buffer must have the same length."
        );
        Self(buffer, mask)
    }

    pub fn fill_with_mask_via<T, F, M>(len: usize, v: F, m: M) -> Self
    where
        T: CellEncoding,
        F: Fn(usize) -> T,
        M: Fn(usize) -> bool,
    {
        let buffer = CellBuffer::fill_via(len, v);
        let mask = Mask::fill_via(len, m);
        Self::new(buffer, mask)
    }

    pub fn buffer(&self) -> &CellBuffer {
        &self.0
    }

    pub fn buffer_mut(&mut self) -> &mut CellBuffer {
        &mut self.0
    }

    pub fn mask(&self) -> &Mask {
        &self.1
    }

    pub fn mask_mut(&mut self) -> &mut Mask {
        &mut self.1
    }

    pub fn get_masked(&self, index: usize) -> Option<CellValue> {
        if self.mask().get(index) {
            Some(self.buffer().get(index))
        } else {
            None
        }
    }

    pub fn get_with_mask(&self, index: usize) -> (CellValue, bool) {
        (self.buffer().get(index), self.mask().get(index))
    }

    /// Convert `self` into a `Vec<T>`, replacing values where the mask is `0` to `no_data.value()`
    pub fn to_vec_with_nodata<T: CellEncoding>(
        self,
        no_data: NoData<T>,
    ) -> crate::error::Result<Vec<T>> {
        let Self(buf, mask) = self;
        let out = buf.to_vec::<T>()?;
        if let Some(no_data) = no_data.value() {
            Ok(out
                .into_iter()
                .zip(mask)
                .map(|(v, m)| if m { v } else { no_data })
                .collect())
        } else {
            Ok(out)
        }
    }
}

impl BufferOps for MaskedCellBuffer {
    fn from_vec<T: CellEncoding>(data: Vec<T>) -> Self {
        let buffer = CellBuffer::from_vec(data);
        let mask = Mask::fill(buffer.len(), true);
        Self::new(buffer, mask)
    }

    fn with_defaults(len: usize, ct: CellType) -> Self {
        let buffer = CellBuffer::with_defaults(len, ct);
        let mask = Mask::fill(len, true);
        Self::new(buffer, mask)
    }

    fn fill(len: usize, value: CellValue) -> Self {
        let buffer = CellBuffer::fill(len, value);
        let mask = Mask::fill(len, true);
        Self::new(buffer, mask)
    }

    fn fill_via<T, F>(len: usize, f: F) -> Self
    where
        T: CellEncoding,
        F: Fn(usize) -> T,
    {
        let buffer = CellBuffer::fill_via(len, f);
        let mask = Mask::fill(len, true);
        Self::new(buffer, mask)
    }

    fn len(&self) -> usize {
        self.buffer().len()
    }

    fn cell_type(&self) -> CellType {
        self.buffer().cell_type()
    }

    fn get(&self, index: usize) -> CellValue {
        self.buffer().get(index)
    }

    fn put(&mut self, idx: usize, value: CellValue) -> crate::error::Result<()> {
        self.buffer_mut().put(idx, value)
    }

    fn convert(&self, cell_type: CellType) -> crate::error::Result<Self>
    where
        Self: Sized,
    {
        let converted = self.buffer().convert(cell_type)?;
        Ok(Self::new(converted, self.mask().to_owned()))
    }

    fn min_max(&self) -> (CellValue, CellValue) {
        let init = (self.cell_type().max(), self.cell_type().min());
        self.into_iter().fold(init, |(amin, amax), (v, m)| {
            if m {
                (amin.min(v), amax.max(v))
            } else {
                (amin, amax)
            }
        })
    }

    /// Converts `self` to `Vec<T>` with default NoData value.
    ///
    /// Replacing cells with corresponding mask value of `0` to
    /// [`NoData<T>::Default`].
    ///
    /// See also: [`Self::to_vec_with_nodata`] and [`NoData`].
    fn to_vec<T: CellEncoding>(self) -> crate::error::Result<Vec<T>> {
        self.to_vec_with_nodata(NoData::Default)
    }
}

impl Debug for MaskedCellBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let basename = self.cell_type().to_string();
        f.debug_tuple(&format!("{basename}MaskedCellBuffer"))
            .field(self.buffer())
            .field(self.mask())
            .finish()
    }
}

impl From<MaskedCellBuffer> for (CellBuffer, Mask) {
    fn from(value: MaskedCellBuffer) -> Self {
        (value.0, value.1)
    }
}

impl<'a> From<&'a MaskedCellBuffer> for (&'a CellBuffer, &'a Mask) {
    fn from(value: &'a MaskedCellBuffer) -> Self {
        (&value.0, &value.1)
    }
}

/// Converts a [`CellBuffer`] into a [`MaskedCellBuffer`] with an all-true mask.
impl From<CellBuffer> for MaskedCellBuffer {
    fn from(value: CellBuffer) -> Self {
        let len = value.len();
        Self::new(value, Mask::fill(len, true))
    }
}

impl<C: CellEncoding> FromIterator<C> for MaskedCellBuffer {
    fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
        Self::from_vec(iter.into_iter().collect())
    }
}

impl<C: CellEncoding> FromIterator<(C, bool)> for MaskedCellBuffer {
    fn from_iter<T: IntoIterator<Item = (C, bool)>>(iter: T) -> Self {
        let (data, mask): (CellBuffer, Mask) = iter.into_iter().unzip();
        Self::new(data, mask)
    }
}

impl<C: CellEncoding> Extend<(C, bool)> for MaskedCellBuffer {
    fn extend<T: IntoIterator<Item = (C, bool)>>(&mut self, iter: T) {
        for (v, m) in iter {
            self.buffer_mut().extend(Some(v));
            self.mask_mut().extend(Some(m));
        }
    }
}

impl<'buf> IntoIterator for &'buf MaskedCellBuffer {
    type Item = (CellValue, bool);
    type IntoIter = MaskedCellBufferIterator<'buf>;

    fn into_iter(self) -> Self::IntoIter {
        MaskedCellBufferIterator { buf: self, idx: 0, len: self.len() }
    }
}

pub struct MaskedCellBufferIterator<'buf> {
    buf: &'buf MaskedCellBuffer,
    idx: usize,
    len: usize,
}

impl Iterator for MaskedCellBufferIterator<'_> {
    type Item = (CellValue, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            None
        } else {
            let r = self.buf.get_with_mask(self.idx);
            self.idx += 1;
            Some(r)
        }
    }
}

mod ops {
    use crate::{CellValue, MaskedCellBuffer};
    use std::ops::{Add, Div, Mul, Neg, Sub};

    macro_rules! cb_bin_op {
        ($trt:ident, $mth:ident, $op:tt) => {
            // Both borrows.
            impl $trt for &MaskedCellBuffer {
                type Output = MaskedCellBuffer;
                fn $mth(self, rhs: Self) -> Self::Output {
                    let (lbuf, lmask) = self.into();
                    let (rbuf, rmask) = rhs.into();
                    let new_buf = lbuf.into_iter().zip(rbuf.into_iter()).map(|(l, r)| l $op r).collect();
                    #[allow(clippy::suspicious_arithmetic_impl)]
                    let new_mask = lmask & rmask;
                    Self::Output::new(new_buf, new_mask)
                }
            }
            // Both owned/consumed
            impl $trt for MaskedCellBuffer {
                type Output = MaskedCellBuffer;
                fn $mth(self, rhs: Self) -> Self::Output {
                    $trt::$mth(&self, &rhs)
                }
            }
            // RHS borrow
            impl $trt<&MaskedCellBuffer> for MaskedCellBuffer {
                type Output = MaskedCellBuffer;
                fn $mth(self, rhs: &MaskedCellBuffer) -> Self::Output {
                    $trt::$mth(&self, &rhs)
                }
            }
            // RHS scalar
            // TODO: figure out how to implement LHS scalar, avoiding orphan rule.
            impl<R> $trt<R> for MaskedCellBuffer
            where
                R: Into<CellValue>,
            {
                type Output = MaskedCellBuffer;
                fn $mth(self, rhs: R) -> Self::Output {
                    let r: CellValue = rhs.into();
                    let (buf, mask) = self.into();
                    let new_buf = buf.into_iter().map(|l | l $op r).collect();
                    Self::new(new_buf, mask)
                }
            }
        };
    }
    cb_bin_op!(Add, add, +);
    cb_bin_op!(Sub, sub, -);
    cb_bin_op!(Mul, mul, *);
    cb_bin_op!(Div, div, /);

    impl Neg for &MaskedCellBuffer {
        type Output = MaskedCellBuffer;
        fn neg(self) -> Self::Output {
            Self::Output::new(self.buffer().neg(), self.mask().clone())
        }
    }
    impl Neg for MaskedCellBuffer {
        type Output = MaskedCellBuffer;
        fn neg(self) -> Self::Output {
            Self::Output::new(self.buffer().neg(), self.1)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BufferOps, CellBuffer, Mask, MaskedCellBuffer, NoData};

    fn filler(i: usize) -> u8 {
        i as u8
    }
    fn masker(i: usize) -> bool {
        i % 2 == 0
    }
    #[test]
    fn ctor() {
        let m = MaskedCellBuffer::fill_via(3, filler);
        let r = MaskedCellBuffer::new(CellBuffer::fill_via(3, filler), Mask::fill(3, true));
        assert_eq!(m, r);
    }

    #[test]
    fn get_masked() {
        let buf = MaskedCellBuffer::fill_with_mask_via(9, filler, masker);
        assert_eq!(buf.get_masked(4), Some(4.into()));
        assert_eq!(buf.get_masked(5), None);
    }

    #[test]
    fn extend() {
        let mut buf = MaskedCellBuffer::fill(3, 0.into());
        buf.extend([(1, false)]);
        assert_eq!(buf.get_masked(0), Some(0.into()));
        assert_eq!(buf.get_masked(3), None);
    }

    #[test]
    fn unary() {
        let mbuf = MaskedCellBuffer::fill_with_mask_via(9, filler, masker);
        let r = -mbuf;
        let v = r.to_vec_with_nodata::<i16>(NoData::Default).unwrap();

        #[rustfmt::skip]
        assert_eq!(
            v,
            vec![0, i16::MIN, -2, i16::MIN, -4, i16::MIN, -6, i16::MIN, -8]
        );
    }

    #[test]
    fn min_max() {
        let mbuf = MaskedCellBuffer::fill_with_mask_via(9, filler, |i| i != 0 && i != 8);
        assert_eq!(mbuf.min_max(), (1u8.into(), 7u8.into()));
    }

    #[test]
    fn scalar() {
        // All `true` case
        let mbuf = MaskedCellBuffer::fill_with_mask_via(9, filler, |_| true);
        let r = mbuf * 2.0;
        let expected = CellBuffer::fill_via(9, filler) * 2.0;
        assert_eq!(r, expected.clone().into());

        // Alternating mask case
        let mbuf = MaskedCellBuffer::fill_with_mask_via(9, filler, masker);
        let r = mbuf * 2.0;
        assert_ne!(r, expected.into());

        let v = r
            .to_vec_with_nodata::<f64>(NoData::Value(f64::MIN))
            .unwrap();

        #[rustfmt::skip]
        assert_eq!(
            v,
            vec![0.0, f64::MIN, 4.0, f64::MIN, 8.0, f64::MIN, 12.0, f64::MIN, 16.0]
        );
    }
}