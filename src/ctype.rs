/*
 * Copyright (c) 2023. Astraea, Inc. All rights reserved.
 */

use crate::error::Error;
use crate::CellValue;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CellType {
    UInt8,
    UInt16,
    // Int16,
    // Int32,
    Float32,
    Float64,
}

impl Display for CellType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl FromStr for CellType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UInt8" => Ok(Self::UInt8),
            "UInt16" => Ok(Self::UInt16),
            "Float32" => Ok(Self::Float32),
            "Float64" => Ok(Self::Float64),
            o => Err(Error::ParseError(o.into(), "CellType")),
        }
    }
}

impl CellType {
    /// Get an iterator over all the valid enumeration values.
    pub fn iter() -> impl Iterator<Item = CellType> {
        use CellType::*;
        [UInt8, UInt16, Float32, Float64].into_iter()
    }

    pub fn is_integral(&self) -> bool {
        match self {
            CellType::UInt8 => true,
            CellType::UInt16 => true,
            CellType::Float32 => false,
            CellType::Float64 => false,
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            CellType::UInt8 => false,
            CellType::UInt16 => false,
            CellType::Float32 => true,
            CellType::Float64 => true,
        }
    }

    /// Select the `CellType` that can numerically contain both `self` and `other`.
    pub fn union(self, other: Self) -> Self {
        // Want to do this, but am afraid it's fragile....
        // if (self as u8) > (other as u8) { self } else { other }
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
        }
    }

    pub fn min(&self) -> CellValue {
        match self {
            CellType::UInt8 => CellValue::UInt8(u8::MIN),
            CellType::UInt16 => CellValue::UInt16(u16::MIN),
            CellType::Float32 => CellValue::Float32(f32::MIN),
            CellType::Float64 => CellValue::Float64(f64::MIN),
        }
    }

    pub fn max(&self) -> CellValue {
        match self {
            CellType::UInt8 => CellValue::UInt8(u8::MAX),
            CellType::UInt16 => CellValue::UInt16(u16::MAX),
            CellType::Float32 => CellValue::Float32(f32::MAX),
            CellType::Float64 => CellValue::Float64(f64::MAX),
        }
    }
}

pub trait HasCellType {
    fn cell_type(&self) -> CellType;
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
    fn can_string() {
        assert_eq!(CellType::Float32.to_string(), "Float32");

        for ct in CellType::iter() {
            let stringed = ct.to_string();
            let parsed = CellType::from_str(&stringed);
            assert!(parsed.is_ok(), "{stringed}");
            assert_eq!(parsed.unwrap(), ct, "{stringed}");
        }

        assert!(CellType::from_str("UInt57").is_err());
    }
}
