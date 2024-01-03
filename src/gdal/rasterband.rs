use super::with_gdal_ct;
use crate::error::*;
use crate::{gdal::GdalND, BufferOps, CellBuffer, CellType, MaskedCellBuffer};
use gdal::raster::{RasterBand, ResampleAlg};

/// Extension methods on [`RasterBand`]  to read/write [`CellBuffer`]s.
pub trait RasterBandEx {
    #[cfg_attr(docsrs, doc(cfg(feature = "gdal")))]
    /// Read a [`CellBuffer`] from a GDAL [`RasterBand`].
    ///
    /// Ignores any no-data value. See [`read_cells_masked`][Self::read_cells_masked] for reading [`MaskedCellBuffer`].
    ///
    /// # Arguments
    /// * `window` - the window position from top left
    /// * `window_size` - the window size (GDAL will interpolate data if `window_size` != `buffer_size`)
    /// * `buffer_size` - the desired size of the 'Buffer'
    /// * `e_resample_alg` - the resample algorithm used for the interpolation. Default: `NearestNeighbor`.
    ///
    /// # Example
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use erased_cells_testkit::DATA_DIR;
    /// use erased_cells::*;
    /// use gdal::Dataset;
    /// use gdal::raster::StatisticsMinMax;
    /// use num_traits::ToPrimitive;
    /// let ds = Dataset::open(DATA_DIR.join("L8-Elkton-VA-B5.tiff"))?;
    /// let rb = ds.rasterband(1)?;
    /// let size = ds.raster_size();
    /// let buffer = rb.read_cells((0, 0), size, size, None)?;
    /// let StatisticsMinMax { min, max } = rb.compute_raster_min_max(false)?;
    /// let (buf_min, buf_max) = buffer.min_max();
    /// assert_eq!((buf_min.to_f64().unwrap(), buf_max.to_f64().unwrap()), (min, max));
    /// # Ok(())
    /// # }
    /// ```
    fn read_cells(
        &self,
        window: (isize, isize),
        window_size: (usize, usize),
        size: (usize, usize),
        e_resample_alg: Option<ResampleAlg>,
    ) -> Result<CellBuffer>;

    #[cfg_attr(docsrs, doc(cfg(feature = "gdal")))]
    /// Read a [`MaskedCellBuffer`] from a GDAL [`RasterBand`].
    ///
    /// Any no-data value setting will be used to construct the buffer's [`Mask`][crate::Mask].
    ///
    /// # Arguments
    /// * `window` - the window position from top left
    /// * `window_size` - the window size (GDAL will interpolate data if `window_size` != `buffer_size`)
    /// * `buffer_size` - the desired size of the 'Buffer'
    /// * `e_resample_alg` - the resample algorithm used for the interpolation. Default: `NearestNeighbor`.
    ///
    /// # Example
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use erased_cells_testkit::DATA_DIR;
    /// use erased_cells::*;
    /// use gdal::Dataset;
    ///
    /// let ds = Dataset::open(DATA_DIR.join("L8-Elkton-VA-B5-nd.tiff"))?;
    /// let rb = ds.rasterband(1)?;
    /// let size = ds.raster_size();
    /// let buffer = rb.read_cells_masked((0, 0), size, size, None)?;
    /// let (data_cnt, nodata_cnt) = buffer.counts();
    /// assert_eq!(data_cnt + nodata_cnt, size.0 * size.1);
    /// # Ok(())
    /// # }
    /// ```
    fn read_cells_masked(
        &self,
        window: (isize, isize),
        window_size: (usize, usize),
        size: (usize, usize),
        e_resample_alg: Option<ResampleAlg>,
    ) -> Result<MaskedCellBuffer>;
}

impl RasterBandEx for RasterBand<'_> {
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
                    o => Err(Error::UnsupportedCellTypeError(o.to_string())),
                }
            }
        }
        with_gdal_ct!(read_cells)
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
                    o => Err(Error::UnsupportedCellTypeError(o.cell_type().to_string())),
                }
            }
        }
        with_gdal_ct!(read_masked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BufferOps, CellBuffer, MaskedCellBuffer};
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
        // gdal_calc.py --calc '(A.astype(double)-B.astype(double))/(A.astype(double)+B.astype(double))' -A testkit/data/L8-Elkton-VA-B5-nd.tiff -B testkit/data/L8-Elkton-VA-B4.tiff --outfile ndvi.tiff --type Float64 --hideNoData
        assert!(min.to_f64().unwrap() - -0.1248899911993 < 1e-8);
        assert!(max.to_f64().unwrap() - 0.66998345719859 < 1e-8);

        Ok(())
    }
}
