//! `erased-cells` connector to GDAL I/O

use crate::{CellBuffer, MaskedCellBuffer};
use extend::ext;
use gdal::raster::{RasterBand, ResampleAlg};
use gdal::Dataset;

#[ext]
impl RasterBand<'_> {
    fn read_cells(
        &self,
        window: (isize, isize),
        window_size: (usize, usize),
        size: (usize, usize),
        e_resample_alg: Option<ResampleAlg>,
    ) -> gdal::errors::Result<CellBuffer> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use erased_cells_testkit::*;
    use gdal::Dataset;
    test_error!(gdal::errors::GdalError);

    #[test]
    fn read_cells() -> TestResult {
        let ds = Dataset::open(DATA_DIR.join("L8-Elkton-VA-B5-nd.tiff"))?;
        let rb = ds.rasterband(1)?;

        rb.read_cells()
    }
}
