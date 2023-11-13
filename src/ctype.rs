/*
 * Copyright (c) 2023. Astraea, Inc. All rights reserved.
 */

use crate::error::Error;
use crate::{with_ct, CellValue};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::mem;
use std::str::FromStr;

// CellType enum constructor.
macro_rules! cv_enum {
    ( $(($id:ident, $_p:ident)),*) => {
        /// Cell-type variants
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[repr(u8)]
        pub enum CellType { $($id),* }
    }
}
with_ct!(cv_enum);

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

    /// Determine if `self` is integral or floating-point.
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

    /// Determine if `self` is signed or unsigned.
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

    /// Number of bytes needed to encode `self`.
    pub fn size_of(&self) -> usize {
        macro_rules! size_of {
            ($( ($id:ident, $p:ident) ),* ) => {
                match self {
                    $(CellType::$id => mem::size_of::<$p>()),*
                }
            };
        }
        with_ct!(size_of)
    }

    /// Select the `CellType` that can numerically contain both `self` and `other`.
    pub fn union(self, other: Self) -> Self {
        let min_bytes = {
            match (self.is_integral(), other.is_integral()) {
                (true, false) => other.size_of().max(2 * self.size_of()),
                (false, true) => self.size_of().max(2 * other.size_of()),
                _ => match (self.is_signed(), other.is_signed()) {
                    (true, false) => self.size_of().max(2 * other.size_of()),
                    (false, true) => other.size_of().max(2 * self.size_of()),
                    _ => self.size_of().max(other.size_of()),
                },
            }
        };
        let signed = self.is_signed() || other.is_signed();
        let integral = self.is_integral() && other.is_integral();
        //dbg!(min_bytes, signed, integral);
        match (min_bytes, signed, integral) {
            (1, false, true) => Self::UInt8,
            (1, true, true) => Self::Int8,
            (2, false, true) => Self::UInt16,
            (2, true, true) => Self::Int16,
            (4, false, true) => Self::UInt32,
            (4, true, true) => Self::Int32,
            (4, _, false) => Self::Float32,
            (8, false, true) => Self::UInt64,
            (8, true, true) => Self::Int64,
            (_, _, false) => Self::Float64,
            _ => unreachable!(
                "No union for {self} & {other}: bytes={min_bytes}, signed={signed}, integral={integral}"
            ),
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
    use crate::{with_ct, CellType};
    use std::str::FromStr;

    #[test]
    fn can_union() {
        // reflexivity
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

        // symmetry
        assert_eq!(CellType::Int16.union(CellType::Float32), CellType::Float32);
        assert_eq!(CellType::Float32.union(CellType::Int16), CellType::Float32);
        // widening
        assert_eq!(CellType::UInt8.union(CellType::UInt16), CellType::UInt16);
        assert_eq!(CellType::Int32.union(CellType::Float32), CellType::Float64);
    }

    #[test]
    fn is_integral() {
        assert!(CellType::UInt8.is_integral());
        assert!(CellType::UInt16.is_integral());
        assert!(!CellType::Float32.is_integral());
        assert!(!CellType::Float64.is_integral());
    }

    #[test]
    fn size() {
        assert_eq!(CellType::Int8.size_of(), 1);
        assert_eq!(CellType::UInt8.size_of(), 1);
        assert_eq!(CellType::Int16.size_of(), 2);
        assert_eq!(CellType::UInt16.size_of(), 2);
        assert_eq!(CellType::Int32.size_of(), 4);
        assert_eq!(CellType::UInt32.size_of(), 4);
        assert_eq!(CellType::Int64.size_of(), 8);
        assert_eq!(CellType::UInt64.size_of(), 8);
        assert_eq!(CellType::Float32.size_of(), 4);
        assert_eq!(CellType::Float64.size_of(), 8);
    }

    #[test]
    fn has_min_max() {
        // Confirm min/max returns correct values.
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
        // Confirm simple serialization.
        macro_rules! test {
            ( $( ($ct:ident, $p:ident) ),* ) => {
                $( assert_eq!(CellType::$ct.to_string(), stringify!($ct)); )*
            };
        }
        with_ct!(test);

        // Test round-trip conversion to/from String
        for ct in CellType::iter() {
            let stringed = ct.to_string();
            let parsed = CellType::from_str(&stringed);
            assert!(parsed.is_ok(), "{stringed}");
            assert_eq!(parsed.unwrap(), ct, "{stringed}");
        }

        assert!(CellType::from_str("UInt57").is_err());
    }
}
