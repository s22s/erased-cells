`with_ct` is a [callback style](https://danielkeep.github.io/tlborm/book/pat-callbacks.html)
macro used to construct various implementations covering all [`CellType`]s.

It calls the passed identifier as a macro with two parameters:
* the cell type id (e.g. `UInt8`),
* the cell type primitive (e.g. `u8`).

# Example
```rust
use erased_cells::{with_ct, CellType};
fn primitive_name(ct: CellType) -> &'static str {
    macro_rules! primitive_name {
       ($(($id:ident, $p:ident)),*) => {
            match ct {
                $(CellType::$id => stringify!($p),)*
            }
       };
    }
    with_ct!(primitive_name)
}

assert_eq!(primitive_name(CellType::Float32), "f32");
```