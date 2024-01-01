//! `erased-cells` connector to GDAL I/O

use crate::error::Result;
use crate::{with_ct, CellEncoding, CellType, NoData};
use gdal::raster::GdalDataType;
use gdal_sys::GDALDataType;
use num_traits::ToPrimitive;
use paste::paste;
use std::ffi::c_uint;

mod rasterband;
pub use rasterband::*;

/// Convert from [`GdalDataType`] to appropriate [`CellType`].
impl TryFrom<GdalDataType> for CellType {
    type Error = crate::error::Error;

    fn try_from(value: GdalDataType) -> Result<Self, Self::Error> {
        match value as c_uint {
            GDALDataType::GDT_Byte => Ok(CellType::UInt8),
            GDALDataType::GDT_UInt16 => Ok(CellType::UInt16),
            GDALDataType::GDT_Int16 => Ok(CellType::Int16),
            GDALDataType::GDT_UInt32 => Ok(CellType::UInt32),
            GDALDataType::GDT_Int32 => Ok(CellType::Int32),
            GDALDataType::GDT_Float32 => Ok(CellType::Float32),
            GDALDataType::GDT_Float64 => Ok(CellType::Float64),
            // Because GDAL < 3.5 is still very common, we are hard-coding
            // the ordinal values so as to simplify  the rest of the code.
            //
            // GDT_UInt64: 64 bit unsigned integer (GDAL >= 3.5)
            12 => Ok(CellType::UInt64),
            // GDT_Int64: 64 bit signed integer  (GDAL >= 3.5)
            13 => Ok(CellType::Int64),
            // GDT_Int8: 8-bit signed integer (GDAL >= 3.7)
            14 => Ok(CellType::Int8),
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
            let _ct: CellType = dt.try_into().unwrap();
        }
    }
}
