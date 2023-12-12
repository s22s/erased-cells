use crate::Elided;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::ops::{BitAnd, BitOr, Index, IndexMut, Not};
use std::vec::IntoIter;

/// Encodes the bit-mask for [`MaskedCellBuffer`][super::MaskedCellBuffer]
#[derive(Default, Clone, PartialEq, PartialOrd, Ord, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Mask(Vec<bool>);

impl Mask {
    /// Create a new mask containing `values`.
    pub fn new(values: Vec<bool>) -> Self {
        Self(values)
    }

    /// Create a mask of size `len` with all values set to `value`.
    pub fn fill(len: usize, value: bool) -> Self {
        Self(vec![value; len])
    }

    /// Create a mask of size `len` were each value is determined by
    /// the value returned by `f`, which is called with the index position as an
    /// argument.
    pub fn fill_via<F>(len: usize, f: F) -> Self
    where
        F: Fn(usize) -> bool,
    {
        Self((0..len).map(f).collect())
    }

    /// Get the number of mask elements.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if `self.len() == 0`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Set the mask value at position `index` to `value`.
    ///
    /// # Panics
    /// Will panic if `index` >= `self.len()`.
    pub fn put(&mut self, index: usize, value: bool) {
        self.0[index] = value;
    }

    /// Get the mask value at position `index`.
    ///
    /// # Panics
    /// Will panic if `index` >= `self.len()`.
    pub fn get(&self, index: usize) -> bool {
        self.0[index]
    }

    /// Gets an mutable iterator over values in mask, in sequence.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &'_ mut bool> {
        self.0.iter_mut()
    }

    /// Determine if all mask values equal `value`.
    pub fn all(&self, value: bool) -> bool {
        self.0.iter().all(|b| *b == value)
    }

    /// Returns a tuple of representing counts of `(data, nodata)`.
    pub fn counts(&self) -> (usize, usize) {
        self.0.iter().fold((0, 0), |(data, nodata), m| {
            if *m {
                (data + 1, nodata)
            } else {
                (data, nodata + 1)
            }
        })
    }
}

impl Extend<bool> for Mask {
    fn extend<T: IntoIterator<Item = bool>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl Index<usize> for Mask {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Mask {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Not for Mask {
    type Output = Mask;
    fn not(mut self) -> Self::Output {
        self.0.iter_mut().for_each(|b| *b = !*b);
        self
    }
}

impl Not for &Mask {
    type Output = Mask;
    fn not(self) -> Self::Output {
        Mask(self.0.iter().map(|b| !*b).collect())
    }
}

impl BitAnd for Mask {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self.0
            .iter_mut()
            .zip(rhs.0.iter())
            .for_each(|(l, r)| *l &= *r);
        self
    }
}

impl BitAnd for &Mask {
    type Output = Mask;
    fn bitand(self, rhs: Self) -> Self::Output {
        Mask(
            self.0
                .iter()
                .zip(rhs.0.iter())
                .map(|(l, r)| *l & *r)
                .collect(),
        )
    }
}

impl BitOr for Mask {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self.0
            .iter_mut()
            .zip(rhs.0.iter())
            .for_each(|(l, r)| *l |= *r);
        self
    }
}

impl BitOr for &Mask {
    type Output = Mask;
    fn bitor(self, rhs: Self) -> Self::Output {
        Mask(
            self.0
                .iter()
                .zip(rhs.0.iter())
                .map(|(l, r)| *l | *r)
                .collect(),
        )
    }
}
impl Debug for Mask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Mask({:?})", Elided(&self.0)))
    }
}

impl IntoIterator for Mask {
    type Item = bool;
    type IntoIter = IntoIter<bool>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::Mask;

    #[test]
    fn counts() {
        let m = Mask::fill(3, true);
        assert_eq!(m.counts(), (3, 0));
        let m = Mask::fill(3, false);
        assert_eq!(m.counts(), (0, 3));
        let m = Mask::fill_via(3, |i| i % 2 == 0);
        assert_eq!(m.counts(), (2, 1));
    }

    #[test]
    fn set() {
        let mut m = Mask::fill(3, true);
        m.put(1, false);
        m[0] = false;
        assert_eq!(m, Mask::new(vec![false, false, true]));
    }

    #[test]
    fn not() {
        let t = Mask::fill(4, true);
        let f = Mask::fill(4, false);
        assert_eq!(!&t, f);
        assert_eq!(!t, f);

        let m = Mask::new(vec![true, false, true, false]);
        let r = Mask::new(vec![false, true, false, true]);
        assert_eq!(!&m, r);
        assert_eq!(!m, r);
    }

    #[test]
    fn all() {
        let m = Mask::fill_via(4, |i| i % 2 == 0);
        assert!(!m.all(true));
        assert!(!m.all(false));
        let m = Mask::fill(4, true);
        assert!(m.all(true));
        assert!(!m.all(false));
    }

    #[test]
    fn and() {
        let l = Mask::fill_via(4, |i| i % 2 == 0);
        let r = Mask::fill_via(4, |i| i % 2 != 0);
        // non-consuming
        assert!((&l & &r).all(false));
        // consuming
        assert!((l & r).all(false));
    }

    #[test]
    fn or() {
        let l = Mask::fill_via(4, |i| i % 2 == 0);
        let r = Mask::fill_via(4, |i| i % 2 != 0);
        // non-consuming
        assert!((&l | &r).all(true));
        // consuming
        assert!((l | r).all(true));
    }
}
