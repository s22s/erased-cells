//! `erased-cells` connector to GDAL I/O
//!
//! # Example
//! ```rust
//!  
//! ```

use crate::error::Result;
use crate::{with_ct, CellBuffer, CellEncoding, CellType, MaskedCellBuffer, NoData};
use extend::ext;
use gdal::raster::{GdalDataType, RasterBand, ResampleAlg};
use num_traits::ToPrimitive;
use paste::paste;

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

struct GdalND(Option<f64>);

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

#[ext]
impl RasterBand<'_> {
    fn read_cells(
        &self,
        window: (isize, isize),
        window_size: (usize, usize),
        size: (usize, usize),
        e_resample_alg: Option<ResampleAlg>,
    ) -> Result<CellBuffer> {
        let ct: CellType = self.band_type().try_into()?;
        macro_rules! read_cells {
            ($( ($id:ident, $p:ident) ),*) => {
                match ct {
                    $(
                    CellType::$id => {
                        let v = self.read_as::<$p>(window, window_size, size, e_resample_alg)?;
                        Ok(CellBuffer::new(v.data))
                    }),*
                }
            }
        }
        with_ct!(read_cells)
    }
    fn read_cells_masked(
        &self,
        window: (isize, isize),
        window_size: (usize, usize),
        size: (usize, usize),
        e_resample_alg: Option<ResampleAlg>,
    ) -> Result<MaskedCellBuffer> {
        let buf = self.read_cells(window, window_size, size, e_resample_alg)?;
        let nd = GdalND(self.no_data_value());

        macro_rules! read_masked {
            ($( ($id:ident, $p:ident) ),*) => {
                match buf {
                    $(
                    CellBuffer::$id(v) => Ok(MaskedCellBuffer::from_vec_with_nodata::<$p>(v, nd.try_into()?)),
                    )*
                }
            }
        }
        with_ct!(read_masked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BufferOps;
    use erased_cells_testkit::*;
    use gdal::Dataset;
    use num_traits::ToPrimitive;
    use std::path::PathBuf;

    #[test]
    fn read_cells() -> TestResult {
        fn read(p: PathBuf) -> Result<CellBuffer> {
            let ds = Dataset::open(p)?;
            let rb = ds.rasterband(1)?;
            rb.read_cells((0, 0), ds.raster_size(), ds.raster_size(), None)
        }

        let red = read(DATA_DIR.join("L8-Elkton-VA-B4.tiff"))?;
        let nir = read(DATA_DIR.join("L8-Elkton-VA-B5.tiff"))?;

        let ndvi = (&nir - &red) / (nir + red);

        // Compare against:
        // gdal_calc.py --calc '(A.astype(double)-B.astype(double))/(A.astype(double)+B.astype(double))' -A testkit/data/L8-Elkton-VA-B5.tiff -B testkit/data/L8-Elkton-VA-B4.tiff --outfile ndvi.tiff --type Float64 --hideNoData
        //     STATISTICS_MAXIMUM=0.66998345719859
        //     STATISTICS_MEAN=0.45559234941397
        //     STATISTICS_MINIMUM=-0.1248899911993
        //     STATISTICS_STDDEV=0.10447748270797
        //     STATISTICS_VALID_PERCENT=100

        let (min, max) = ndvi.min_max();
        assert!(min.to_f64().unwrap() - -0.1248899911993 < 1e-8);
        assert!(max.to_f64().unwrap() - 0.66998345719859 < 1e-8);

        Ok(())
    }

    #[test]
    fn read_cells_masked() -> TestResult {
        fn read(p: PathBuf) -> Result<MaskedCellBuffer> {
            let ds = Dataset::open(p)?;
            let rb = ds.rasterband(1)?;
            rb.read_cells_masked((0, 0), ds.raster_size(), ds.raster_size(), None)
        }

        let red = read(DATA_DIR.join("L8-Elkton-VA-B4.tiff"))?;
        let nir = read(DATA_DIR.join("L8-Elkton-VA-B5-nd.tiff"))?;

        let (nir_data, nir_nodata) = nir.counts();

        let ndvi = (&nir - &red) / (nir + red);

        // The NIR band has 4 nodata values in it, as should the result.
        let (ndvi_data, ndvi_nodata) = ndvi.counts();
        assert_eq!(nir_data, ndvi_data);
        assert_eq!(nir_nodata, ndvi_nodata);

        let (min, max) = ndvi.min_max();
        dbg!(min, max);

        // gdal_calc.py --calc '(A.astype(double)-B.astype(double))/(A.astype(double)+B.astype(double))' -A testkit/data/L8-Elkton-VA-B5-nd.tiff -B testkit/data/L8-Elkton-VA-B4.tiff --outfile ndvi.tiff --type Float64 --hideNoData
        assert!(min.to_f64().unwrap() - -0.1248899911993 < 1e-8);
        assert!(max.to_f64().unwrap() - 0.66998345719859 < 1e-8);

        Ok(())
    }
}
