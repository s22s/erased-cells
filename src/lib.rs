/*
 * Copyright (c) 2023. Astraea, Inc. All rights reserved.
 */

//! Encoding and manipulation of runtime-dynamic raster cell values.

mod buffer;
mod ctype;
mod encoding;
pub mod error;
mod value;

pub use buffer::ops::*;
pub use buffer::*;
pub use ctype::*;
pub use encoding::*;
pub use value::ops::*;
pub use value::*;

/// A [callback style](https://danielkeep.github.io/tlborm/book/pat-callbacks.html)
/// macro used to construct various implementations in this crate.
macro_rules! with_ct {
    ($callback:ident) => {
        $callback! {
            (UInt8, u8),
            (UInt16, u16),
            (UInt32, u32),
            (UInt64, u64),
            (Int8, i8),
            (Int16, i16),
            (Int32, i32),
            (Int64, i64),
            (Float32, f32),
            (Float64, f64)
        }
    };
}
pub(crate) use with_ct;
