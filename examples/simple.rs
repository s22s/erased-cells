use erased_cells::{CellBuffer, CellType, CellValue};

fn main() {
    // Fill a buffer with the `u8` numbers `0..=8`.
    let buf1 = CellBuffer::fill_with(9, |i| i as u8);

    // `u8` maps to `CellType::UInt8`
    assert_eq!(buf1.cell_type(), CellType::UInt8);

    // A fetching values maintains its CellType through a CellValue.
    let val: CellValue = buf1.get(3);
    assert_eq!(val, CellValue::UInt8(3));
    let (min, max): (CellValue, CellValue) = buf1.minmax();
    assert_eq!((min, max), (CellValue::UInt8(0), CellValue::UInt8(8)));

    // Basic math ops work on CellValues. Primitives can be converted to CellValues with `into`.
    // Math ops coerce to floating point values.
    assert_eq!(((max - min + 1.into()) / 2.into()), 4.5.into());

    // Fill another buffer with the `f32` numbers `8..=0`.
    let buf2 = CellBuffer::fill_with(9, |i| 8f32 - i as f32);
    assert_eq!(buf2.cell_type(), CellType::Float32);
    assert_eq!(
        buf2.minmax(),
        (CellValue::Float32(0.0), CellValue::Float32(8.0))
    );

    // Basic math ops also work on CellBuffers, applied element-wise.
    let diff = buf2 - buf1;
    assert_eq!(diff.minmax(), ((-8).into(), 8.into()));
}
