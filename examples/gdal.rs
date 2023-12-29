#[cfg(feature = "gdal")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use erased_cells::*;
    use erased_cells_testkit::DATA_DIR;
    use gdal::*;

    // Un-masked
    {
        use gdal::raster::StatisticsMinMax;
        let ds = Dataset::open(DATA_DIR.join("L8-Elkton-VA-B5-nd.tiff"))?;
        let rb = ds.rasterband(1)?;
        let size = ds.raster_size();
        let buffer = rb.read_cells((0, 0), size, size, None)?;
        let StatisticsMinMax { min, max } = rb.compute_raster_min_max(false)?;
        let min_max = buffer.min_max();
        assert_eq!(min_max, (min.into(), max.into()));
    }

    // Masked
    {
        let ds = Dataset::open(DATA_DIR.join("L8-Elkton-VA-B5-nd.tiff"))?;
        let rb = ds.rasterband(1)?;
        let size = ds.raster_size();
        let buffer = rb.read_cells_masked((0, 0), size, size, None)?;
        let (data_cnt, nodata_cnt) = buffer.counts();
        assert_eq!(data_cnt + nodata_cnt, size.0 * size.1);
    }
    Ok(())
}

#[cfg(not(feature = "gdal"))]
fn main() {}
