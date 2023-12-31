# Erased Buffers
[![Build Status]][actions] [![Test Coverage]][codecov] [![Latest Version]][crates.io] [![Documentation]][docs.rs] 

[Build Status]: https://github.com/s22s/erased-cells/actions/workflows/CI.yml/badge.svg
[actions]: https://github.com/s22s/erased-cells/actions?query=branch%3Adevelop
[Latest Version]: https://img.shields.io/crates/v/erased-cells.svg
[crates.io]: https://crates.io/crates/erased-cells
[Test Coverage]: https://codecov.io/gh/s22s/erased-cells/graph/badge.svg?token=6GKU96IMV5
[codecov]: https://codecov.io/gh/s22s/erased-cells
[Documentation]: https://img.shields.io/docsrs/erased-cells
[docs.rs]: https://docs.rs/erased-cells/latest/erased_cells/

Enables the use and manipulation of type-erased buffers of Rust primitives.

Please refer to the [documentation](https://docs.rs/erased-cells/) for details.

## Quick Example

```rust
use erased_cells::CellBuffer;

fn main() {
    // Create a buffer with u8 values.
    let buf1 = CellBuffer::from(vec![1u8, 2, 3]);
    // Create a buffer with u16 values.
    let buf2 = CellBuffer::from(vec![2u16, 4, 6]);
    // Perform element-wise and scalar math. Division coerces buffer to f64. 
    let result = buf1 / buf2 * 0.5;
    // Expected result:
    assert_eq!(result, vec![0.25, 0.25, 0.25].into());
}
```

See [here in the documentation](https://docs.rs/erased-cells/latest/erased_cells/#examples) for additional examples.