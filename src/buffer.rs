use std::fmt::{Debug, Formatter};

use num_traits::ToPrimitive;
use paste::paste;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::{with_ct, BufferOps, CellEncoding, CellType, CellValue};

pub use self::ops::*;

/// CellBuffer enum constructor.
macro_rules! cb_enum {
    ( $(($id:ident, $p:ident)),*) => {
        /// An enum over buffers of [`CellEncoding`] types.
        ///
        /// See [module documentation](crate#example) for usage example.
        ///
        /// # Example
        ///
        /// ```rust
        /// use erased_cells::{CellBuffer, CellType, CellValue, BufferOps};
        /// // Fill a buffer with the `u8` numbers `0..=8`.
        /// let buf1 = CellBuffer::fill_via(9, |i| i as u8);
        ///
        /// // `u8` maps to `CellType::UInt8`
        /// assert_eq!(buf1.cell_type(), CellType::UInt8);
        ///
        /// // A fetching values maintains its CellType through a CellValue.
        /// let val: CellValue = buf1.get(3);
        /// assert_eq!(val, CellValue::UInt8(3));
        /// let (min, max): (CellValue, CellValue) = buf1.min_max();
        /// assert_eq!((min, max), (CellValue::UInt8(0), CellValue::UInt8(8)));
        ///
        /// // Basic math ops work on CellValues. Primitives can be converted to CellValues with `into`.
        /// // Math ops coerce to floating point values.
        /// assert_eq!(((max - min + 1) / 2), 4.5.into());
        ///
        /// // Fill another buffer with the `f32` numbers `8..=0`.
        /// let buf2 = CellBuffer::fill_via(9, |i| 8f32 - i as f32);
        /// assert_eq!(buf2.cell_type(), CellType::Float32);
        /// assert_eq!(
        ///     buf2.min_max(),
        ///     (CellValue::Float32(0.0), CellValue::Float32(8.0))
        /// );
        ///
        /// // Basic math ops also work on CellBuffers, applied element-wise.
        /// let diff = buf2 - buf1;
        /// assert_eq!(diff.min_max(), ((-8).into(), 8.into()));
        /// ```
        #[derive(Clone)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        pub enum CellBuffer { $($id(Vec<$p>)),* }
    }
}
with_ct!(cb_enum);

impl CellBuffer {
    pub fn new<T: CellEncoding>(data: Vec<T>) -> Self {
        data.into()
    }
}

impl BufferOps for CellBuffer {
    fn from_vec<T: CellEncoding>(data: Vec<T>) -> Self {
        data.into()
    }

    fn with_defaults(len: usize, ct: CellType) -> Self {
        macro_rules! empty {
            ( $(($id:ident, $p:ident)),*) => {
                match ct {
                    $(CellType::$id => Self::from_vec(vec![$p::default(); len]),)*
                }
            };
        }
        with_ct!(empty)
    }

    fn fill(len: usize, value: CellValue) -> Self {
        macro_rules! empty {
            ( $(($id:ident, $p:ident)),*) => {
                match value.cell_type() {
                    $(CellType::$id => Self::from_vec::<$p>(vec![value.get().unwrap(); len]),)*
                }
            };
        }
        with_ct!(empty)
    }

    fn fill_via<T, F>(len: usize, f: F) -> Self
    where
        T: CellEncoding,
        F: Fn(usize) -> T,
    {
        let v: Vec<T> = (0..len).map(f).collect();
        Self::from_vec(v)
    }

    fn len(&self) -> usize {
        macro_rules! len {
            ( $(($id:ident, $_p:ident)),*) => {
                match self {
                    $(CellBuffer::$id(v) => v.len(),)*
                }
            };
        }
        with_ct!(len)
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn cell_type(&self) -> CellType {
        macro_rules! ct {
            ( $(($id:ident, $_p:ident)),*) => {
                match self {
                    $(CellBuffer::$id(_) => CellType::$id,)*
                }
            };
        }
        with_ct!(ct)
    }

    fn get(&self, index: usize) -> CellValue {
        macro_rules! get {
            ( $(($id:ident, $_p:ident)),*) => {
                match self {
                    $(CellBuffer::$id(b) => CellValue::$id(b[index]),)*
                }
            };
        }
        with_ct!(get)
    }

    fn put(&mut self, idx: usize, value: CellValue) -> Result<()> {
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

    fn convert(&self, cell_type: CellType) -> Result<Self> {
        if cell_type == self.cell_type() {
            return Ok(self.clone());
        }

        let err = || Error::NarrowingError { src: self.cell_type(), dst: cell_type };

        if !self.cell_type().can_fit_into(cell_type) {
            return Err(err());
        }

        let r: CellBuffer = self
            .into_iter()
            .map(|v| v.convert(cell_type).unwrap())
            .collect();

        Ok(r)
    }

    fn min_max(&self) -> (CellValue, CellValue) {
        let init = (self.cell_type().max_value(), self.cell_type().min_value());
        self.into_iter()
            .fold(init, |(amin, amax), v| (amin.min(v), amax.max(v)))
    }

    fn to_vec<T: CellEncoding>(self) -> Result<Vec<T>> {
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
        use crate::Elided;
        let basename = self.cell_type().to_string();
        macro_rules! render {
            ( $(($id:ident, $_p:ident)),*) => {{
                f.write_fmt(format_args!("{basename}CellBuffer("))?;
                match self {
                    $(CellBuffer::$id(b) => f.write_fmt(format_args!("{:?}", Elided(b)))?,)*
                };
                f.write_str(")")
            }}
        }
        with_ct!(render)
    }
}

impl<C: CellEncoding> Extend<C> for CellBuffer {
    fn extend<T: IntoIterator<Item = C>>(&mut self, iter: T) {
        macro_rules! render {
            ( $(($id:ident, $p:ident)),*) => { paste! {
                match self {
                    $(CellBuffer::$id(b) => {
                        let conv_iter = iter.into_iter().map(|c| {
                            c.into_cell_value().[<to_ $p>]().unwrap()
                        });
                        b.extend(conv_iter)
                    },)*
                }
            }}
        }
        with_ct!(render);
    }
}

impl<C: CellEncoding> FromIterator<C> for CellBuffer {
    fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
        Self::from_vec(iter.into_iter().collect())
    }
}

impl FromIterator<CellValue> for CellBuffer {
    fn from_iter<T: IntoIterator<Item = CellValue>>(iterable: T) -> Self {
        // TODO: is there a way to avoid this collect?
        let values = iterable.into_iter().collect::<Vec<CellValue>>();
        match values.as_slice() {
            [] => CellBuffer::with_defaults(0, CellType::UInt8),
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

/// Iterator over [`CellValue`] elements in a [`CellBuffer`].
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

impl<C: CellEncoding> TryFrom<CellBuffer> for Vec<C> {
    type Error = Error;

    fn try_from(value: CellBuffer) -> Result<Self> {
        value.to_vec()
    }
}

mod ops {
    use std::cmp::Ordering;
    use std::ops::{Add, Div, Mul, Neg, Sub};

    use crate::{BufferOps, CellBuffer, CellValue};

    macro_rules! cb_bin_op {
        ($trt:ident, $mth:ident, $op:tt) => {
            // Both borrows.
            impl $trt for &CellBuffer {
                type Output = CellBuffer;
                fn $mth(self, rhs: Self) -> Self::Output {
                    self.into_iter().zip(rhs.into_iter()).map(|(l, r)| l $op r).collect()
                }
            }
            // Both owned/consumed
            impl $trt for CellBuffer {
                type Output = CellBuffer;
                fn $mth(self, rhs: Self) -> Self::Output {
                    $trt::$mth(&self, &rhs)
                }
            }
            // RHS borrow
            impl $trt<&CellBuffer> for CellBuffer {
                type Output = CellBuffer;
                fn $mth(self, rhs: &CellBuffer) -> Self::Output {
                    $trt::$mth(&self, &rhs)
                }
            }
            // RHS scalar
            // TODO: figure out how to implement LHS scalar, avoiding orphan rule.
            impl <R> $trt<R> for CellBuffer where R: Into<CellValue> {
                type Output = CellBuffer;
                fn $mth(self, rhs: R) -> Self::Output {
                    let r: CellValue = rhs.into();
                    self.into_iter().map(|l | l $op r).collect()
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
            self.into_iter().map(|v| -v).collect()
        }
    }
    impl Neg for CellBuffer {
        type Output = CellBuffer;
        fn neg(self) -> Self::Output {
            Neg::neg(&self)
        }
    }

    impl PartialEq<Self> for CellBuffer {
        fn eq(&self, other: &Self) -> bool {
            Ord::cmp(self, other) == Ordering::Equal
        }
    }

    impl Eq for CellBuffer {}

    impl PartialOrd for CellBuffer {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    /// Computes ordering for [`CellBuffer`]. Unlike `Vec<CellEncoding>`, floating point
    /// cell types are compared with `{f32|f64}::total_cmp`.  
    impl Ord for CellBuffer {
        fn cmp(&self, other: &Self) -> Ordering {
            let lct = self.cell_type();
            let rct = other.cell_type();

            // If the cell types are different, then base comparison on that.
            match lct.cmp(&rct) {
                Ordering::Equal => (),
                ne => return ne,
            }

            macro_rules! total_cmp {
                ($l:ident, $r:ident) => {{
                    // Implementation derived from core::slice::cmp::SliceOrd.
                    let len = $l.len().min($r.len());

                    // Slice to the loop iteration range to enable bound check
                    // elimination in the compiler
                    let lhs = &$l[..len];
                    let rhs = &$r[..len];

                    for i in 0..len {
                        match lhs[i].total_cmp(&rhs[i]) {
                            Ordering::Equal => (),
                            non_eq => return non_eq,
                        }
                    }
                    $l.len().cmp(&$r.len())
                }};
            }
            // lhs & rhs have to be the same variant.
            // For integral values, defer to `Vec`'s `Ord`.
            // For floating values, we use `total_ord`.
            match (self, other) {
                (CellBuffer::UInt8(lhs), CellBuffer::UInt8(rhs)) => Ord::cmp(&lhs, &rhs),
                (CellBuffer::UInt16(lhs), CellBuffer::UInt16(rhs)) => Ord::cmp(&lhs, &rhs),
                (CellBuffer::UInt32(lhs), CellBuffer::UInt32(rhs)) => Ord::cmp(&lhs, &rhs),
                (CellBuffer::UInt64(lhs), CellBuffer::UInt64(rhs)) => Ord::cmp(&lhs, &rhs),
                (CellBuffer::Int8(lhs), CellBuffer::Int8(rhs)) => Ord::cmp(&lhs, &rhs),
                (CellBuffer::Int16(lhs), CellBuffer::Int16(rhs)) => Ord::cmp(&lhs, &rhs),
                (CellBuffer::Int32(lhs), CellBuffer::Int32(rhs)) => Ord::cmp(&lhs, &rhs),
                (CellBuffer::Int64(lhs), CellBuffer::Int64(rhs)) => Ord::cmp(&lhs, &rhs),
                (CellBuffer::Float32(l), CellBuffer::Float32(r)) => total_cmp!(l, r),
                (CellBuffer::Float64(l), CellBuffer::Float64(r)) => total_cmp!(l, r),
                _ => unreachable!("{self:?} <> {other:?}"),
            }
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
    use crate::{with_ct, BufferOps, CellBuffer, CellType, CellValue};

    fn bigger(start: CellType) -> impl Iterator<Item = CellType> {
        CellType::iter().filter(move |ct| start.can_fit_into(*ct))
    }

    #[test]
    fn defaults() {
        macro_rules! test {
            ($( ($id:ident, $p:ident) ),*) => {
                $({
                    let cv = CellBuffer::with_defaults(3, CellType::$id);
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
                    let mut cv = CellBuffer::fill(3, <$p>::default().into());
                    let one = CellValue::new(<$p>::one());
                    cv.put(1, one).expect("Put one");
                    assert_eq!(cv.get(1), one.convert(CellType::$id).unwrap());
                })*};
        }
        with_ct!(test);
    }

    #[test]
    fn extend() {
        let mut buf = CellBuffer::fill(3, 0u8.into());
        assert!(!buf.is_empty());
        assert_eq!(buf.cell_type(), CellType::UInt8);
        buf.extend([1]);
        assert_eq!(buf.cell_type(), CellType::UInt8);
        assert_eq!(buf.get(0), 0.into());
        assert_eq!(buf.get(3), 1.into());
    }

    #[test]
    fn to_vec() {
        macro_rules! test {
            ($( ($id:ident, $p:ident) ),*) => {
                $({
                    let v = vec![<$p>::default(); 3];
                    let buf = CellBuffer::from_vec(v.clone());
                    let r = buf.to_vec::<$p>().unwrap();
                    assert_eq!(r, v);
                })*
            };
        }
        with_ct!(test);
    }

    #[test]
    fn min_max() {
        let buf = CellBuffer::from_vec(vec![-1.0, 3.0, 2000.0, -5555.5]);
        let (min, max) = buf.min_max();
        assert_eq!(min, CellValue::Float64(-5555.5));
        assert_eq!(max, CellValue::Float64(2000.0));

        let buf = CellBuffer::from_vec(vec![1u8, 3u8, 200u8, 0u8]);
        let (min, max) = buf.min_max();
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
        let b = CellBuffer::fill(5, 37.into());
        assert!(format!("{b:?}").starts_with("Int32CellBuffer"));
        let b = CellBuffer::fill(15, 37.into());
        assert!(format!("{b:?}").contains("..."));
    }

    #[test]
    fn convert() {
        for ct in CellType::iter() {
            let buf = CellBuffer::with_defaults(3, ct);
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
                let buf = -CellBuffer::fill(3, one);
                assert_eq!(buf.get(0), -one);
            })*};
        }

        with_ct!(test);
    }

    #[test]
    fn binary() {
        for lhs_ct in CellType::iter() {
            let lhs_val = lhs_ct.one();
            for rhs_ct in CellType::iter() {
                let lhs = CellBuffer::fill(3, lhs_val);
                let rhs_val = rhs_ct.one() + rhs_ct.one();
                let rhs = CellBuffer::fill(3, rhs_val);
                assert_eq!((&lhs + &rhs).get(0), lhs_val + rhs_val);
                assert_eq!((&rhs + &lhs).get(1), rhs_val + lhs_val);
                assert_eq!((&lhs - &rhs).get(2), lhs_val - rhs_val);
                assert_eq!((&rhs - &lhs).get(0), rhs_val - lhs_val);
                assert_eq!((&lhs * &rhs).get(1), lhs_val * rhs_val);
                assert_eq!((&rhs * &lhs).get(2), rhs_val * lhs_val);
                assert_eq!((&lhs / &rhs).get(0), lhs_val / rhs_val);
                assert_eq!((&rhs / &lhs).get(1), rhs_val / lhs_val);
                // Consuming (non-borrow) case
                assert_eq!((rhs / lhs).get(2), rhs_val / lhs_val);
            }
        }
    }

    #[test]
    fn scalar() {
        let buf = CellBuffer::fill_via(9, |i| i as u8 + 1);
        let r = buf * 2.0;
        assert_eq!(r, CellBuffer::fill_via(9, |i| (i as f64 + 1.0) * 2.0));
    }

    #[test]
    fn equal() {
        let buf = CellBuffer::fill_via(9, |i| if i % 2 == 0 { f64::NAN } else { i as f64 });
        assert_eq!(buf, buf);
        assert_eq!(
            CellBuffer::with_defaults(4, CellType::UInt8),
            CellBuffer::with_defaults(4, CellType::UInt8)
        );
        assert_ne!(
            CellBuffer::with_defaults(4, CellType::UInt8),
            CellBuffer::with_defaults(5, CellType::UInt8)
        );
    }

    #[test]
    fn cmp() {
        assert!(CellBuffer::from_vec(vec![1, 2, 3]) < CellBuffer::from_vec(vec![2, 3, 4]));
        // Higher positioned elements determine ordering
        assert!(CellBuffer::from_vec(vec![1, 2, 3]) < CellBuffer::from_vec(vec![2, 3]));

        assert!(
            CellBuffer::from_vec(vec![f64::NAN, 2.0, 3.0])
                < CellBuffer::from_vec(vec![f64::NAN, 2.0, 4.0])
        );

        assert!(
            CellBuffer::with_defaults(4, CellType::UInt8)
                < CellBuffer::with_defaults(4, CellType::Float32)
        );
        assert!(
            CellBuffer::with_defaults(4, CellType::Float32)
                > CellBuffer::with_defaults(4, CellType::UInt8)
        );
        assert!(
            CellBuffer::with_defaults(4, CellType::UInt8)
                < CellBuffer::with_defaults(5, CellType::UInt8)
        );
        assert!(
            CellBuffer::with_defaults(5, CellType::UInt8)
                > CellBuffer::with_defaults(4, CellType::UInt8)
        );
        assert!(
            CellBuffer::with_defaults(4, CellType::Float64)
                < CellBuffer::with_defaults(5, CellType::Float64)
        );
        assert!(
            CellBuffer::with_defaults(5, CellType::Float64)
                > CellBuffer::with_defaults(4, CellType::Float64)
        );
    }
}
