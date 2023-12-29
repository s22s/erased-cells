//! `erased-cells` connector to GDAL I/O

use crate::error::Result;
use crate::{with_ct, CellEncoding, CellType, NoData};
use gdal::raster::GdalDataType;
use num_traits::ToPrimitive;
use paste::paste;

mod rasterband;
pub use rasterband::*;

/// Convert from [`GdalDataType`] to appropriate [`CellType`].
impl TryFrom<GdalDataType> for CellType {
    type Error = crate::error::Error;

    fn try_from(value: GdalDataType) -> Result<Self, Self::Error> {
        match value {
            GdalDataType::UInt8 => Ok(CellType::UInt8),
            GdalDataType::UInt16 => Ok(CellType::UInt16),
            GdalDataType::Int16 => Ok(CellType::Int16),
            GdalDataType::UInt32 => Ok(CellType::UInt32),
            GdalDataType::Int32 => Ok(CellType::Int32),
            GdalDataType::Float32 => Ok(CellType::Float32),
            GdalDataType::Float64 => Ok(CellType::Float64),
            o => Err(Self::Error::UnsupportedCellTypeError(o.to_string())),
        }
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
            let ct: CellType = dt.try_into().unwrap();
        }
    }
}
