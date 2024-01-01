//! `erased-cells` connector to GDAL I/O

use crate::error::Result;
use crate::{with_ct, CellEncoding, CellType, NoData};
use gdal::raster::GdalDataType;
use num_traits::ToPrimitive;
use paste::paste;

mod rasterband;
pub use rasterband::*;

// Note: Older versions of GDAL do not support Int8, Int64 and UInt64, so we have
// a reduced set of cell types in this module.
macro_rules! with_gdal_ct {
    ($callback:ident) => {
        $callback! {
            (UInt8, u8),
            (UInt16, u16),
            (UInt32, u32),
            (Int16, i16),
            (Int32, i32),
            (Float32, f32),
            (Float64, f64)
        }
    };
}
pub(crate) use with_gdal_ct;

/// Convert from [`GdalDataType`] to appropriate [`CellType`].
impl TryFrom<GdalDataType> for CellType {
    type Error = crate::error::Error;

    fn try_from(value: GdalDataType) -> Result<Self, Self::Error> {
        macro_rules! try_from {
            ($( ($id:ident, $_p:ident) ),*) => {
                match value {
                    $(GdalDataType::$id => Ok(CellType::$id),)*
                    o => Err(Self::Error::UnsupportedCellTypeError(o.to_string())),
                }
            }
        }
        with_gdal_ct!(try_from)
    }
}

/// No-data conversion support.
pub(crate) struct GdalND(Option<f64>);

impl<T: CellEncoding> TryFrom<GdalND> for NoData<T> {
    type Error = crate::error::Error;

    fn try_from(value: GdalND) -> std::result::Result<Self, Self::Error> {
        macro_rules! nd_convert {
            ($( ($id:ident, $p:ident) ),*) => { paste! {
                match value.0 {
                    None => Ok(NoData::None),
                    Some(nd) => match T::cell_type() {
                        $(CellType::$id => nd
                            .[<to_ $p>]()
                            .and_then(T::static_cast)
                            .map(NoData::new)
                            .ok_or(Self::Error::NoDataConversionError(nd, stringify!($p)))
                        ,)*
                    }
                }
            }}
        }
        with_ct!(nd_convert)
    }
}

#[cfg(test)]
mod tests {
    use crate::CellType;
    use gdal::raster::GdalDataType;

    #[test]
    fn gdal_enum() {
        for dt in GdalDataType::iter().filter(|g| !format!("{g:?}").contains("Int64")) {
            let _ct: CellType = dt.try_into().unwrap();
        }
    }
}
