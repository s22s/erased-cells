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

fn main() {
    assert_eq!(primitive_name(CellType::Float32), "f32");
}
