/*
 * Copyright (c) 2023. Astraea, Inc. All rights reserved.
 */

use crate::error::Error;
use crate::CellValue;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CellType {
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
}

/// `Display` is the same as `Debug`.
impl Display for CellType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl FromStr for CellType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        macro_rules! str_match {
            ( $( ($ct:ident, $_p:ident) ),* ) => {
                match s {
                    $( stringify!($ct) => Ok(CellType::$ct), )*
                    o => Err(Error::ParseError(o.into(), "CellType")),
                }
            };
        }
        with_ct!(str_match)
    }
}

impl CellType {
    /// Get an iterator over all the valid enumeration values.
    pub fn iter() -> impl Iterator<Item = CellType> {
        use CellType::*;
        macro_rules! ct_array {
           ( $( ($id:ident, $_p:ident) ),+) => { [ $( $id, )+ ] };
        }
        with_ct!(ct_array).into_iter()
    }

    pub fn is_integral(&self) -> bool {
        match self {
            CellType::UInt8 => true,
            CellType::UInt16 => true,
            CellType::UInt32 => true,
            CellType::UInt64 => true,
            CellType::Int8 => true,
            CellType::Int16 => true,
            CellType::Int32 => true,
            CellType::Int64 => true,
            CellType::Float32 => false,
            CellType::Float64 => false,
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            CellType::UInt8 => false,
            CellType::UInt16 => false,
            CellType::UInt32 => false,
            CellType::UInt64 => false,
            CellType::Int8 => true,
            CellType::Int16 => true,
            CellType::Int32 => true,
            CellType::Int64 => true,
            CellType::Float32 => true,
            CellType::Float64 => true,
        }
    }

    /// Select the `CellType` that can numerically contain both `self` and `other`.
    pub fn union(self, other: Self) -> Self {
        // Want to do this, but am afraid it's fragile:
        //     if (self as u8) > (other as u8) { self } else { other }
        match self {
            CellType::UInt8 => other,
            CellType::UInt16 => match other {
                CellType::UInt8 => self,
                _ => other,
            },
            CellType::Float32 => match other {
                CellType::Float64 => other,
                _ => self,
            },
            CellType::Float64 => self,
            _ => todo!(),
        }
    }

    pub fn min(&self) -> CellValue {
        macro_rules! mins {
            ( $( ($ct:ident, $p:ident) ),* ) => {
                match self {
                    $( CellType::$ct => CellValue::$ct($p::MIN), )*
                }
            };
        }
        with_ct!(mins)
    }

    pub fn max(&self) -> CellValue {
        macro_rules! maxs {
            ( $( ($ct:ident, $p:ident) ),* ) => {
                match self {
                    $( CellType::$ct => CellValue::$ct($p::MAX), )*
                }
            };
        }
        with_ct!(maxs)
    }
}

#[cfg(test)]
mod tests {
    use crate::CellType;
    use std::str::FromStr;

    #[test]
    fn can_union() {
        assert_eq!(CellType::UInt8.union(CellType::UInt8), CellType::UInt8);
        assert_eq!(CellType::UInt16.union(CellType::UInt16), CellType::UInt16);
        assert_eq!(
            CellType::Float32.union(CellType::Float32),
            CellType::Float32
        );
        assert_eq!(
            CellType::Float64.union(CellType::Float64),
            CellType::Float64
        );
    }

    #[test]
    fn is_integral() {
        assert!(CellType::UInt8.is_integral());
        assert!(CellType::UInt16.is_integral());
        assert!(!CellType::Float32.is_integral());
        assert!(!CellType::Float64.is_integral());
    }

    #[test]
    fn has_min_max() {
        macro_rules! test {
            ( $( ($ct:ident, $p:ident) ),* ) => {
                $(
                    assert_eq!(CellType::$ct.min(), $p::MIN.into(), "min");
                    assert_eq!(CellType::$ct.max(), $p::MAX.into(), "max");
                )*
            };
        }
        with_ct!(test);
    }

    #[test]
    fn can_string() {
        macro_rules! test {
            ( $( ($ct:ident, $p:ident) ),* ) => {
                $( assert_eq!(CellType::$ct.to_string(), stringify!($ct)); )*
            };
        }
        with_ct!(test);

        for ct in CellType::iter() {
            let stringed = ct.to_string();
            let parsed = CellType::from_str(&stringed);
            assert!(parsed.is_ok(), "{stringed}");
            assert_eq!(parsed.unwrap(), ct, "{stringed}");
        }

        assert!(CellType::from_str("UInt57").is_err());
    }
}
