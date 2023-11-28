use crate::{with_ct, BufferOps, CellBuffer, CellEncoding, CellType, CellValue, Elided, NoData};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::vec::IntoIter;

#[derive(Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Mask(Vec<bool>);

impl Mask {
    pub fn fill(value: bool, len: usize) -> Self {
        Self(vec![value; len])
    }
    pub fn len(&self) -> usize {
        self.0.len()
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

/// MaskedCellBuffer enum constructor.
macro_rules! cb_enum {
    ( $(($id:ident, $p:ident)),*) => {
        #[derive(Clone, PartialEq, PartialOrd)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        /// A cell buffer with cell validity mask.
        pub enum MaskedCellBuffer { $($id(CellBuffer, Mask)),* }
    }
}
with_ct!(cb_enum);

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
        macro_rules! new {
            ($(($id:ident, $p:ident)),*) => {
                match buffer.cell_type() {
                    $(CellType::$id => Self::$id(buffer, mask)),*
                }
            };
        }

        with_ct!(new)
    }

    pub fn buffer(&self) -> &CellBuffer {
        macro_rules! buffer {
            ($(($id:ident, $p:ident)),*) => {
                match self {
                    $(Self::$id(buffer, _) => &buffer),*
                }
            };
        }

        with_ct!(buffer)
    }

    pub fn buffer_mut(&mut self) -> &mut CellBuffer {
        macro_rules! buffer {
            ($(($id:ident, $p:ident)),*) => {
                match self {
                    $(Self::$id(buffer, _) => &mut *buffer),*
                }
            };
        }

        with_ct!(buffer)
    }

    pub fn mask(&self) -> &Mask {
        macro_rules! mask {
            ($(($id:ident, $p:ident)),*) => {
                match self {
                    $(Self::$id(_, mask) => &mask),*
                }
            };
        }

        with_ct!(mask)
    }

    pub fn mask_mut(&mut self) -> &mut Mask {
        macro_rules! mask {
            ($(($id:ident, $p:ident)),*) => {
                match self {
                    $(Self::$id(_, mask) => &mut *mask),*
                }
            };
        }

        with_ct!(mask)
    }

    /// Separate underlying buffer and mask from `self`.
    // TODO: Better name?
    pub fn take(self) -> (CellBuffer, Mask) {
        macro_rules! take {
            ($(($id:ident, $p:ident)),*) => {
                match self {
                    $(Self::$id(buffer, mask) => (buffer, mask)),*
                }
            };
        }

        with_ct!(take)
    }

    /// Convert `self` into a `Vec<T>`, replacing values where the mask is `0` to `no_data.value()`
    pub fn to_vec_with_nodata<T: CellEncoding>(
        self,
        no_data: NoData<T>,
    ) -> crate::error::Result<Vec<T>> {
        let (buf, mask) = self.take();
        let out = buf.to_vec()?;
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
        let mask = Mask::fill(true, buffer.len());
        Self::new(buffer, mask)
    }

    fn with_defaults(len: usize, ct: CellType) -> Self {
        let buffer = CellBuffer::with_defaults(len, ct);
        let mask = Mask::fill(true, len);
        Self::new(buffer, mask)
    }

    fn fill(value: CellValue, len: usize) -> Self {
        let buffer = CellBuffer::fill(value, len);
        let mask = Mask::fill(true, len);
        Self::new(buffer, mask)
    }

    fn fill_with<T: CellEncoding>(len: usize, f: fn(usize) -> T) -> Self {
        let buffer = CellBuffer::fill_with(len, f);
        let mask = Mask::fill(true, len);
        Self::new(buffer, mask)
    }

    fn len(&self) -> usize {
        self.buffer().len()
    }

    fn cell_type(&self) -> CellType {
        self.buffer().cell_type()
    }

    fn get(&self, idx: usize) -> CellValue {
        self.buffer().get(idx)
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
        self.buffer().min_max()
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

#[cfg(test)]
mod tests {
    use crate::{BufferOps, MaskedCellBuffer};

    #[test]
    fn ctor() {
        let mb = MaskedCellBuffer::fill_with(3, |i| i as u8 + 1);
        println!("{mb:#?}")
    }
}
