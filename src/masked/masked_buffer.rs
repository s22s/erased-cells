use std::fmt::{Debug, Formatter};

pub use self::ops::*;
use crate::masked::nodata::IsNodata;
use crate::{BufferOps, CellBuffer, CellEncoding, CellType, CellValue, Mask, NoData};
use serde::{Deserialize, Serialize};

/// A [`CellBuffer`] with a companion [`Mask`].
///
/// The `Mask` tracks which cells are valid across operations, and which should be
/// treated as "no-data" values.
///
/// # Example
///
/// ```rust
/// use erased_cells::{BufferOps, Mask, MaskedCellBuffer};
/// // Fill a buffer with the `u16` numbers `0..=3` and mask [true, false, true, false].
/// let buf = MaskedCellBuffer::fill_with_mask_via(4, |i| (i as f64, i % 2 == 0));
/// assert_eq!(buf.mask(), &Mask::new(vec![true, false, true, false]));
/// // We can count the data/no-data values
/// assert_eq!(buf.counts(), (2, 2));
///
/// // Mask values are propagated across math operations.
/// let ones = MaskedCellBuffer::from_vec(vec![1.0; 4]);
/// let r = (buf + ones) * 2.0;
///
/// let expected = MaskedCellBuffer::new(
///     vec![
///         (0.0 + 1.0) * 2.0,
///         (1.0 + 1.0) * 2.0,
///         (2.0 + 1.0) * 2.0,
///         (3.0 + 1.0) * 2.0,
///     ]
///     .into(),
///     Mask::new(vec![true, false, true, false]),
/// );
/// assert_eq!(r, expected);
/// ```
#[derive(Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MaskedCellBuffer(CellBuffer, Mask);

impl MaskedCellBuffer {
    /// Create a new combined [`CellBuffer`] and [`Mask`].
    ///
    /// # Panics
    /// Will panics if `buffer` and `mask` are not the same length.
    pub fn new(buffer: CellBuffer, mask: Mask) -> Self {
        assert_eq!(
            buffer.len(),
            mask.len(),
            "Mask and buffer must have the same length."
        );
        Self(buffer, mask)
    }

    /// Constructs a `MaskedCellBuffer` from a `Vec<CellEncoding>`, specifying a `NoData<T>` value.
    ///
    /// Mask value will be `false` when associated cell matches `nodata`.
    ///
    /// Use [`Self::from_vec`]
    pub fn from_vec_with_nodata<T: CellEncoding>(data: Vec<T>, nodata: NoData<T>) -> Self {
        let mut mask = Mask::fill(data.len(), true);
        let buf = CellBuffer::from_vec(data);

        buf.into_iter().zip(mask.iter_mut()).for_each(|(v, m)| {
            *m = !v.is(nodata);
        });

        Self::new(buf, mask)
    }

    pub fn fill_with_mask_via<T, F>(len: usize, mv: F) -> Self
    where
        T: CellEncoding,
        F: Fn(usize) -> (T, bool),
    {
        (0..len).map(mv).collect()
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

    /// Get a buffer value at position `index` with mask evaluated.
    ///
    /// Returns `Some(CellValue)` if mask at `index` is `true`, `None` otherwise.
    pub fn get_masked(&self, index: usize) -> Option<CellValue> {
        if self.mask().get(index) {
            Some(self.buffer().get(index))
        } else {
            None
        }
    }

    /// Get the cell value and mask value at position `index`.
    ///
    /// Returns `(CellValue, bool)`. If `bool` is `false`, associated
    /// `CellValue` should be considered invalid.
    pub fn get_with_mask(&self, index: usize) -> (CellValue, bool) {
        (self.buffer().get(index), self.mask().get(index))
    }

    /// Set the `value` and `mask` at position `index`.
    ///
    /// Returns `Err(NarrowingError)` if `value` cannot be converted to
    /// `self.cell_type()` without data loss (e.g. overflow).
    pub fn put_with_mask(
        &mut self,
        index: usize,
        value: CellValue,
        mask: bool,
    ) -> crate::error::Result<()> {
        self.put(index, value)?;
        self.mask_mut().put(index, mask);
        Ok(())
    }

    /// Returns a tuple of representing counts of `(data, nodata)`.
    pub fn counts(&self) -> (usize, usize) {
        self.mask().counts()
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
        let init = (self.cell_type().max_value(), self.cell_type().min_value());
        self.into_iter().fold(init, |(amin, amax), (v, m)| {
            if m {
                (amin.min(v), amax.max(v))
            } else {
                (amin, amax)
            }
        })
    }

    /// Converts `self` to `Vec<T>`, ignoring the `mask` values.
    ///
    /// See also: [`Self::to_vec_with_nodata`] and [`NoData`].
    fn to_vec<T: CellEncoding>(self) -> crate::error::Result<Vec<T>> {
        self.0.to_vec()
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
        // This is basically a copy of `unzip` except that we control
        // instantiation of an empty CellBuffer to ensure the right
        // CellType. Possible because
        // `impl Extend for (impl Extend, impl Extend)` exists.
        let mut pair = (
            CellBuffer::with_defaults(0, C::cell_type()),
            Mask::default(),
        );
        pair.extend(iter);

        let (data, mask) = pair;
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

/// Iterator over ([`CellValue`], `bool`) elements in a [`MaskedCellBuffer`].
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
    use crate::{BufferOps, CellBuffer, CellType, CellValue, Mask, MaskedCellBuffer, NoData};

    fn filler(i: usize) -> u8 {
        i as u8
    }
    fn masker(i: usize) -> bool {
        i % 2 == 0
    }
    fn filler_masker(i: usize) -> (u8, bool) {
        (filler(i), masker(i))
    }

    #[test]
    fn ctor() {
        let m = MaskedCellBuffer::fill_via(3, filler);
        let r = MaskedCellBuffer::new(CellBuffer::fill_via(3, filler), Mask::fill(3, true));
        assert_eq!(m, r);

        let m = MaskedCellBuffer::from_vec(vec![0.0; 4]);
        assert_eq!(m.mask().counts(), (4, 0));
        let m = MaskedCellBuffer::with_defaults(4, CellType::Int8);
        assert_eq!(m.mask().counts(), (4, 0));
    }

    #[test]
    fn vec_with_nodata() {
        let v = vec![1.0, f64::NAN, 3.0, f64::NAN];
        let m = MaskedCellBuffer::from_vec_with_nodata(v.clone(), NoData::Default);
        assert_eq!(
            m,
            MaskedCellBuffer::new(v.clone().into(), Mask::new(vec![true, false, true, false]))
        );
        let m = MaskedCellBuffer::from_vec_with_nodata(v.clone(), NoData::new(3.0));
        assert_eq!(
            m,
            MaskedCellBuffer::new(v.into(), Mask::new(vec![true, true, false, true]))
        );
    }

    #[test]
    fn get_masked() {
        let mut buf = MaskedCellBuffer::fill_with_mask_via(9, filler_masker);
        assert_eq!(buf.get(4), 4.into());
        assert_eq!(buf.get_masked(4), Some(4.into()));
        assert_eq!(buf.get_masked(5), None);
        buf.put(5, CellValue::new(4u8)).unwrap();
        assert_eq!(buf.get_masked(5), None);

        buf.mask_mut().put(5, true);
        assert_eq!(buf.get_masked(5), Some(4.into()));
        buf.put_with_mask(5, CellValue::new(99u8), false).unwrap();
        assert_eq!(buf.get_masked(5), None);
    }

    #[test]
    fn convert() {
        let buf = MaskedCellBuffer::fill_with_mask_via(4, filler_masker);
        let r = buf.convert(CellType::Float64).unwrap();
        assert_eq!(r.to_vec::<f64>().unwrap(), [0.0, 1.0, 2.0, 3.0]);
    }

    #[test]
    fn extend() {
        let mut buf = MaskedCellBuffer::fill(3, 0.into());
        buf.extend([(1, false)]);
        assert_eq!(buf.get_masked(0), Some(0.into()));
        assert_eq!(buf.get_masked(3), None);
    }

    #[test]
    fn from_iter() {
        let buf: MaskedCellBuffer = (0..5i16).collect();
        assert!(buf.mask().all(true));
        assert_eq!(buf.to_vec::<i16>().unwrap(), [0, 1, 2, 3, 4i16]);
    }

    #[test]
    fn unary() {
        let mbuf = MaskedCellBuffer::fill_with_mask_via(9, filler_masker);
        let r = -&mbuf;
        let v = r.to_vec_with_nodata::<i16>(NoData::Default).unwrap();

        #[rustfmt::skip]
        assert_eq!(
            v,
            vec![0, i16::MIN, -2, i16::MIN, -4, i16::MIN, -6, i16::MIN, -8]
        );

        let r = -mbuf;

        assert_eq!(r.to_vec_with_nodata::<i16>(NoData::Default).unwrap(), v);
    }

    #[test]
    fn min_max() {
        let mbuf = MaskedCellBuffer::fill_with_mask_via(9, |i| (filler(i), i != 0 && i != 8));
        assert_eq!(mbuf.min_max(), (1u8.into(), 7u8.into()));
    }

    #[test]
    fn scalar() {
        // All `true` case
        let mbuf = MaskedCellBuffer::fill_with_mask_via(9, |i| (filler(i), true));
        let r = mbuf * 2.0;
        let expected = CellBuffer::fill_via(9, filler) * 2.0;
        assert_eq!(r, expected.clone().into());

        // Alternating mask case
        let mbuf = MaskedCellBuffer::fill_with_mask_via(9, filler_masker);
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

    #[test]
    fn binary() {
        let lhs =
            MaskedCellBuffer::new(CellBuffer::fill(9, 1f64.into()), Mask::fill_via(9, masker));
        let rhs = MaskedCellBuffer::new(CellBuffer::fill(9, 2f64.into()), Mask::fill(9, true));

        macro_rules! test_ops {
            ($($op:tt)*) => {$({
                let r = &lhs $op &rhs;
                assert_eq!(r.get_masked(0), Some((1f64 $op 2f64).into()));
                assert_eq!(r.get_masked(1), None);
                let r = lhs.clone() $op &rhs;
                assert_eq!(r.get_masked(2), Some((1f64 $op 2f64).into()));
                assert_eq!(r.get_masked(3), None);
                let r = lhs.clone() $op rhs.clone();
                assert_eq!(r.get_masked(4), Some((1f64 $op 2f64).into()));
                assert_eq!(r.get_masked(5), None);
            })*};
        }
        test_ops!(+ - * /);
    }

    #[test]
    fn debug() {
        let m: MaskedCellBuffer = (0..1).collect();
        let dbg = format!("{m:?}");
        assert!(dbg.starts_with("Int32MaskedCellBuffer"));
        assert!(dbg.contains("CellBuffer(0)"));
        assert!(dbg.contains("Mask(true)"));
    }
}
