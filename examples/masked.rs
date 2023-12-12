#[cfg(feature = "masked")]
fn main() {
    use erased_cells::{BufferOps, Mask, MaskedCellBuffer};
    // Fill a buffer with the `u16` numbers `0..=3` and mask [true, false, true, false].
    let buf = MaskedCellBuffer::fill_with_mask_via(4, |i| (i as f64, i % 2 == 0));
    assert_eq!(buf.mask(), &Mask::new(vec![true, false, true, false]));
    // We can count the data/no-data values
    assert_eq!(buf.counts(), (2, 2));

    // Mask values are propagated across math operations.
    let ones = MaskedCellBuffer::from_vec(vec![1.0; 4]);
    let r = (buf + ones) * 2.0;
    let expected = MaskedCellBuffer::new(
        vec![
            (0.0 + 1.0) * 2.0,
            (1.0 + 1.0) * 2.0,
            (2.0 + 1.0) * 2.0,
            (3.0 + 1.0) * 2.0,
        ]
        .into(),
        Mask::new(vec![true, false, true, false]),
    );
    assert_eq!(r, expected);
}

#[cfg(not(feature = "masked"))]
fn main() {}
