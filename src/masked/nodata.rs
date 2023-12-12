use crate::{CellEncoding, CellType, CellValue};

/// Encodes a no-data value for cells that should be considered invalid
/// or masked-out of a result.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum NoData<T: CellEncoding> {
    /// Case where there is no no-data value.
    None,
    /// Case where there the default no-data value should be used.
    #[default]
    Default,
    /// Case where a specific no-data value is specified.
    Value(T),
}

impl<T: CellEncoding> NoData<T> {
    pub fn new(value: T) -> Self {
        NoData::Value(value)
    }
    pub fn value(&self) -> Option<T> {
        match self {
            NoData::None => None,
            NoData::Value(v) => Some(*v),
            NoData::Default => match T::cell_type() {
                CellType::UInt8 => T::static_cast(<u8>::MIN),
                CellType::UInt16 => T::static_cast(<u16>::MIN),
                CellType::UInt32 => T::static_cast(<u32>::MIN),
                CellType::UInt64 => T::static_cast(<u64>::MIN),
                CellType::Int8 => T::static_cast(<i8>::MIN),
                CellType::Int16 => T::static_cast(<i16>::MIN),
                CellType::Int32 => T::static_cast(<i32>::MIN),
                CellType::Int64 => T::static_cast(<i64>::MIN),
                CellType::Float32 => T::static_cast(<f32>::NAN),
                CellType::Float64 => T::static_cast(<f64>::NAN),
            },
        }
    }
    pub fn is(&self, value: &CellValue) -> bool {
        if let Some(nd_val) = self.value() {
            let nd_val2 = nd_val.into_cell_value();
            &nd_val2 == value
        } else {
            false
        }
    }
}

pub trait IsNodata {
    /// Determines if the `self` matches given `NoData` value.
    fn is<N: CellEncoding>(&self, no_data: NoData<N>) -> bool;
}

impl IsNodata for CellValue {
    fn is<N: CellEncoding>(&self, no_data: NoData<N>) -> bool {
        no_data.is(self)
    }
}

impl<T: CellEncoding> IsNodata for T {
    fn is<N: CellEncoding>(&self, no_data: NoData<N>) -> bool {
        no_data.is(&self.into_cell_value())
    }
}

#[cfg(test)]
mod tests {
    use crate::{with_ct, IsNodata, NoData};

    #[test]
    fn has_value() {
        assert_eq!(NoData::<i8>::None.value(), None);
        assert_eq!(NoData::<u8>::Default.value(), Some(<u8>::MIN));
        assert!(NoData::<f32>::Default.value().unwrap().is_nan());
        assert_eq!(NoData::new(6u16).value(), Some(6u16));
    }

    #[test]
    fn defaults() {
        macro_rules! test {
            ($( ($id:ident, $p:ident) ),*) => {
                $(assert!(NoData::<$p>::Default.value().is_some());)*
            }
        }
        with_ct!(test);
    }

    #[test]
    fn is_nodata() {
        assert!(f64::NAN.is(NoData::<f64>::Default));
    }
}
