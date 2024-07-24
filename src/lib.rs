//! Newtype wrapper providing a convenient representation of _four-character-code_ values.
//!
//! Using this type in a public APIs as an alternative to simply passing the equivalent `u32`
//! makes the value's expected use explicit.
//!
//!
//! ## Creating a FourCC value
//!
//! ```rust
//! use four_cc::FourCC;
//!
//! let uuid = FourCC(*b"uuid");
//!
//! // using Into
//! let code: FourCC = b"uuid".into();
//! assert_eq!(uuid, code);
//! ```
//!
//! ## From a slice
//!
//! ```rust
//! # use four_cc::FourCC;
//! let data = b"moofftyp";
//! let code = FourCC::from(&data[0..4]);  // would panic if fewer than 4 bytes
//! assert_eq!(FourCC(*b"moof"), code);
//! ```
//!
//! ## From a u32
//!
//! ```rust
//! # use four_cc::FourCC;
//! let data: u32 = 0x6d6f6f66;
//! let code = FourCC::from(data);
//! assert_eq!(FourCC(*b"moof"), code);
//! // conversion back into a u32
//! let converted: u32 = code.into();
//! assert_eq!(data, converted);
//! ```
//!
//! ## Constants
//!
//! FourCC values can be used in const expressions
//!
//! ```rust
//! # use four_cc::FourCC;
//! const UUID: FourCC = FourCC(*b"uuid");
//! ```
//!
//! ## Matching
//!
//! You can use FourCC values in match patterns as long as you define constants to match against,
//!
//! ```rust
//! # use four_cc::FourCC;
//! const UUID: FourCC = FourCC(*b"uuid");
//! const MOOV: FourCC = FourCC(*b"moov");
//! # let other_value = UUID;
//! match other_value {
//!     MOOV => println!("movie"),
//!     UUID => println!("unique identifier"),
//!     // compiler will not accept: FourCC(*b"trun") => println!("track fragment run"),
//!     _ => println!("Other value; scary stuff")
//! }
//! ```
//!
//! ## Invalid literal values
//!
//! If the literal has other than four bytes, compilation will fail
//!
//! ```compile_fail
//! # use four_cc::FourCC;
//! let bad_fourcc = FourCC(*b"uuid123");
//! // -> expected an array with a fixed size of 4 elements, found one with 7 elements
//! ```
//! **Note** the FourCC value _may_ contain non-printable byte values, including the byte-value zero.
//!
//! ## Debug display
//!
//! ```rust
//! # use four_cc::FourCC;
//! # use std::fmt::Debug;
//! let uuid = FourCC(*b"uuid");
//! # assert_eq!("FourCC(uuid)", format!("{:?}", &uuid));
//! println!("it's {:?}", uuid);  // produces: it's FourCC{uuid}
//! ```
//!
//! Note that if the FourCC bytes are not able to be converted to UTF8, then a fallback
//! representation will be used (as it would be surprising for `format!()` to panic).
//!
//! ```rust
//! # use four_cc::FourCC;
//! # use std::fmt::Debug;
//! let uuid = FourCC(*b"u\xFFi\0");
//! # assert_eq!("FourCC(u\\xffi\\x00)", format!("{:?}", &uuid));
//! println!("it's {:?}", uuid);  // produces: it's FourCC{u\xffi\x00}
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, future_incompatible, missing_docs)]
#![cfg_attr(feature = "nightly", feature(const_trait_impl))]
#![cfg_attr(not(feature = "std"), no_std)]

use core::cmp::Ordering;
use core::fmt;
use core::fmt::Write;
use core::result::Result;
use core::str::FromStr;

/// A _four-character-code_ value.
///
/// See the [module level documentation](index.html).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "zerocopy", derive(zerocopy::FromBytes, zerocopy::AsBytes))]
#[repr(C, packed)]
pub struct FourCC(pub [u8; 4]);
impl FourCC {
    const fn from_u32(self: Self) -> u32 {
        ((self.0[0] as u32) << 24 & 0xff000000)
            | ((self.0[1] as u32) << 16 & 0x00ff0000)
            | ((self.0[2] as u32) << 8 & 0x0000ff00)
            | ((self.0[3] as u32) & 0x000000ff)
    }
}
impl<'a> From<&'a [u8; 4]> for FourCC {
    fn from(buf: &[u8; 4]) -> FourCC {
        FourCC([buf[0], buf[1], buf[2], buf[3]])
    }
}
impl<'a> From<&'a [u8]> for FourCC {
    fn from(buf: &[u8]) -> FourCC {
        FourCC([buf[0], buf[1], buf[2], buf[3]])
    }
}
impl From<u32> for FourCC {
    fn from(val: u32) -> FourCC {
        FourCC([
            (val >> 24 & 0xff) as u8,
            (val >> 16 & 0xff) as u8,
            (val >> 8 & 0xff) as u8,
            (val & 0xff) as u8,
        ])
    }
}
impl PartialOrd for FourCC {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Implement comparison logic here, possibly using the inner FourCC value
        // For example, if FourCC can be converted to something comparable:
        self.to_string().partial_cmp(&other.to_string())
    }
}
impl Ord for FourCC {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}
impl FromStr for FourCC {
    type Err = u32;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err(s.len() as u32);
        }
        let mut buf = [0u8; 4];
        buf.copy_from_slice(s.as_bytes());
        Ok(FourCC(buf))
    }
}

// The macro is needed, because the `impl const` syntax doesn't exists on `stable`.
#[cfg(not(feature = "nightly"))]
macro_rules! from_fourcc_for_u32 {
    () => {
        impl From<FourCC> for u32 {
            fn from(val: FourCC) -> Self {
                val.from_u32()
            }
        }
    };
}
#[cfg(feature = "nightly")]
macro_rules! from_fourcc_for_u32 {
    ($($t:tt)*) => {
        impl const From<FourCC> for u32 {
            fn from(val: FourCC) -> Self {
                val.from_u32()
            }
        }
    };
}
from_fourcc_for_u32!();

impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let b = &self.0;
        let iter = core::ascii::escape_default(b[0])
            .chain(core::ascii::escape_default(b[1]))
            .chain(core::ascii::escape_default(b[2]))
            .chain(core::ascii::escape_default(b[3]));
        for c in iter {
            f.write_char(c as char)?;
        }
        Ok(())
    }
}

impl fmt::Debug for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_tuple("FourCC")
            .field(&format_args!("{}", self))
            .finish()
    }
}

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for FourCC {
    fn schema_name() -> String {
        "FourCC".to_string()
    }
    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        gen.subschema_for::<&str>()
    }
    fn is_referenceable() -> bool {
        false
    }
}

#[cfg(feature = "serde")]
impl serde::ser::Serialize for FourCC {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
struct FromStrVisitor<T> {
    expecting: &'static str,
    ty: core::marker::PhantomData<T>,
}

#[cfg(feature = "serde")]
impl<T> FromStrVisitor<T> {
    fn new(expecting: &'static str) -> Self {
        FromStrVisitor {
            expecting: expecting,
            ty: core::marker::PhantomData,
        }
    }
}

#[cfg(feature = "serde")]
impl core::str::FromStr for FourCC {
    type Err = u32;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.as_bytes().into())
    }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::de::Visitor<'de> for FromStrVisitor<T>
where
    T: core::str::FromStr,
    T::Err: fmt::Display,
{
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.expecting)
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserialize<'de> for FourCC {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(FromStrVisitor::new("FourCC"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq() {
        assert_eq!(FourCC(*b"uuid"), b"uuid".into());
        assert_ne!(FourCC(*b"uuid"), b"diuu".into());
    }

    #[test]
    fn int_conversions() {
        let val: u32 = FourCC(*b"ABCD").into();
        assert_eq!(0x41424344_u32, val);
        assert_eq!(FourCC(*b"ABCD"), 0x41424344u32.into());
    }

    #[cfg(feature = "std")]
    #[test]
    fn display() {
        assert_eq!("uuid", format!("{}", FourCC(*b"uuid")));
        assert_eq!("\\x00uid", format!("{}", FourCC(*b"\x00uid")));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize() {
        use serde_test::{assert_tokens, Token};

        let code = FourCC(*b"uuid");
        assert_tokens(&code, &[Token::Str("uuid")]);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize() {
        use std::str::FromStr;
        let data = "uuid";
        let code = FourCC::from_str(data).unwrap();
        assert_eq!(code, FourCC(*b"uuid"));
    }

    #[cfg(feature = "schemars")]
    #[test]
    fn schema() {
        let schema = schemars::schema_for!(FourCC);
        let expected_type = schemars::schema::InstanceType::String;
        assert_eq!(
            schema.schema.instance_type,
            Some(schemars::schema::SingleOrVec::Single(Box::from(
                expected_type
            )))
        );
    }
}
