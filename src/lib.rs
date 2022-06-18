#![warn(missing_docs)]
//! # ulid-rs
//!
//! This is a Rust implementation of the [ulid][ulid] project which provides
//! Universally Unique Lexicographically Sortable Identifiers.
//!
//! [ulid]: https://github.com/ulid/spec
//!
//!
//! ## Quickstart
//!
//! ```rust
//! # use ulid::Ulid;
//! // Generate a ulid
//! # #[cfg(not(feature = "std"))]
//! # let ulid = Ulid::default();
//! # #[cfg(feature = "std")]
//! let ulid = Ulid::new();
//!
//! // Generate a string for a ulid
//! let s = ulid.to_string();
//!
//! // Create from a String
//! let res = Ulid::from_string(&s);
//! assert_eq!(ulid, res.unwrap());
//!
//! // Or using FromStr
//! let res = s.parse();
//! assert_eq!(ulid, res.unwrap());
//!
//! ```
#![cfg_attr(not(feature = "std"), no_std)]

#[doc = include_str!("../README.md")]
#[cfg(all(doctest, feature = "std"))]
struct ReadMeDoctest;

mod base32;
#[cfg(feature = "std")]
mod time;
#[cfg(feature = "std")]
mod generator;
#[cfg(feature = "serde")]
pub mod serde;
#[cfg(feature = "uuid")]
mod uuid;

use core::fmt;
use core::str::FromStr;

pub use crate::base32::{DecodeError, EncodeError, ULID_LEN};
#[cfg(feature = "std")]
pub use crate::generator::{Generator, MonotonicError};

/// Create a right-aligned bitmask of $len bits
macro_rules! bitmask {
    ($len:expr) => {
        ((1 << $len) - 1)
    };
}
// Allow other modules to use the macro
pub(crate) use bitmask;

/// A Ulid is a unique 128-bit lexicographically sortable identifier
///
/// Canonically, it is represented as a 26 character Crockford Base32 encoded
/// string.
///
/// Of the 128-bits, the first 48 are a unix timestamp in milliseconds. The
/// remaining 80 are random. The first 48 provide for lexicographic sorting and
/// the remaining 80 ensure that the identifier is unique.
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Ulid(pub u128);

impl Ulid {
    /// The number of bits in a Ulid's time portion
    pub const TIME_BITS: u8 = 48;
    /// The number of bits in a Ulid's random portion
    pub const RAND_BITS: u8 = 80;

    /// Create a Ulid from separated parts.
    ///
    /// NOTE: Any overflow bits in the given args are discarded
    ///
    /// # Example
    /// ```rust
    /// use ulid::Ulid;
    ///
    /// let ulid = Ulid::from_string("01D39ZY06FGSCTVN4T2V9PKHFZ").unwrap();
    ///
    /// let ulid2 = Ulid::from_parts(ulid.timestamp_ms(), ulid.random());
    ///
    /// assert_eq!(ulid, ulid2);
    /// ```
    pub const fn from_parts(timestamp_ms: u64, random: u128) -> Ulid {
        let time_part = (timestamp_ms & bitmask!(Self::TIME_BITS)) as u128;
        let rand_part = random & bitmask!(Self::RAND_BITS);
        Ulid((time_part << Self::RAND_BITS) | rand_part)
    }

    /// Creates a Ulid from a Crockford Base32 encoded string
    ///
    /// An DecodeError will be returned when the given string is not formated
    /// properly.
    ///
    /// # Example
    /// ```rust
    /// use ulid::Ulid;
    ///
    /// let text = "01D39ZY06FGSCTVN4T2V9PKHFZ";
    /// let result = Ulid::from_string(text);
    ///
    /// assert!(result.is_ok());
    /// assert_eq!(&result.unwrap().to_string(), text);
    /// ```
    pub const fn from_string(encoded: &str) -> Result<Ulid, DecodeError> {
        match base32::decode(encoded) {
            Ok(int_val) => Ok(Ulid(int_val)),
            Err(err) => Err(err),
        }
    }

    /// The 'nil Ulid'.
    ///
    /// The nil Ulid is special form of Ulid that is specified to have
    /// all 128 bits set to zero.
    ///
    /// # Example
    /// ```rust
    /// use ulid::Ulid;
    ///
    /// let ulid = Ulid::nil();
    ///
    /// assert_eq!(
    ///     ulid.to_string(),
    ///     "00000000000000000000000000"
    /// );
    /// ```
    pub const fn nil() -> Ulid {
        Ulid(0)
    }

    /// Gets the timestamp section of this ulid
    ///
    /// # Example
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// use ::time::OffsetDateTime;
    /// use ulid::Ulid;
    ///
    /// let dt = OffsetDateTime::now_utc();
    /// let ulid = Ulid::from_datetime(dt);
    ///
    /// assert_eq!(ulid.timestamp_ms(), (dt.unix_timestamp_nanos() / 1_000_000) as u64);
    /// # }
    /// ```
    pub const fn timestamp_ms(&self) -> u64 {
        (self.0 >> Self::RAND_BITS) as u64
    }

    /// Gets the random section of this ulid
    ///
    /// # Example
    /// ```rust
    /// use ulid::Ulid;
    ///
    /// let text = "01D39ZY06FGSCTVN4T2V9PKHFZ";
    /// let ulid = Ulid::from_string(text).unwrap();
    /// let ulid_next = ulid.increment().unwrap();
    ///
    /// assert_eq!(ulid.random() + 1, ulid_next.random());
    /// ```
    pub const fn random(&self) -> u128 {
        self.0 & bitmask!(Self::RAND_BITS)
    }

    /// Creates a Crockford Base32 encoded string that represents this Ulid
    ///
    /// # Example
    /// ```rust
    /// use ulid::Ulid;
    ///
    /// let text = "01D39ZY06FGSCTVN4T2V9PKHFZ";
    /// let ulid = Ulid::from_string(text).unwrap();
    ///
    /// let mut buf = [0; ulid::ULID_LEN];
    /// let new_text = ulid.to_str(&mut buf).unwrap();
    ///
    /// assert_eq!(new_text, text);
    /// ```
    pub fn to_str<'buf>(&self, buf: &'buf mut [u8]) -> Result<&'buf mut str, EncodeError> {
        let len = base32::encode_to(self.0, buf)?;
        Ok(unsafe { core::str::from_utf8_unchecked_mut(&mut buf[..len]) })
    }

    /// Creates a Crockford Base32 encoded string that represents this Ulid
    ///
    /// # Example
    /// ```rust
    /// use ulid::Ulid;
    ///
    /// let text = "01D39ZY06FGSCTVN4T2V9PKHFZ";
    /// let ulid = Ulid::from_string(text).unwrap();
    ///
    /// assert_eq!(&ulid.to_string(), text);
    /// ```
    #[allow(clippy::inherent_to_string_shadow_display)] // Significantly faster than Display::to_string
    #[cfg(feature = "std")]
    pub fn to_string(&self) -> String {
        base32::encode(self.0)
    }

    /// Test if the Ulid is nil
    ///
    /// # Example
    /// ```rust
    /// use ulid::Ulid;
    ///
    /// # #[cfg(not(feature = "std"))]
    /// # let ulid = Ulid(1);
    /// # #[cfg(feature = "std")]
    /// let ulid = Ulid::new();
    /// assert!(!ulid.is_nil());
    ///
    /// let nil = Ulid::nil();
    /// assert!(nil.is_nil());
    /// ```
    pub const fn is_nil(&self) -> bool {
        self.0 == 0u128
    }

    /// Increment the random number, make sure that the ts millis stays the same
    pub const fn increment(&self) -> Option<Ulid> {
        const MAX_RANDOM: u128 = bitmask!(Ulid::RAND_BITS);

        if (self.0 & MAX_RANDOM) == MAX_RANDOM {
            None
        } else {
            Some(Ulid(self.0 + 1))
        }
    }
}

impl Default for Ulid {
    fn default() -> Self {
        Ulid::nil()
    }
}

#[cfg(feature = "std")]
impl From<Ulid> for String {
    fn from(ulid: Ulid) -> String {
        ulid.to_string()
    }
}

impl From<(u64, u64)> for Ulid {
    fn from((msb, lsb): (u64, u64)) -> Self {
        Ulid(u128::from(msb) << 64 | u128::from(lsb))
    }
}

impl From<Ulid> for (u64, u64) {
    fn from(ulid: Ulid) -> (u64, u64) {
        ((ulid.0 >> 64) as u64, (ulid.0 & bitmask!(64)) as u64)
    }
}

impl From<u128> for Ulid {
    fn from(value: u128) -> Ulid {
        Ulid(value)
    }
}

impl From<Ulid> for u128 {
    fn from(ulid: Ulid) -> u128 {
        ulid.0
    }
}

impl FromStr for Ulid {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ulid::from_string(s)
    }
}

impl fmt::Display for Ulid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let mut buffer = [0; ULID_LEN];
        write!(f, "{}", self.to_str(&mut buffer).unwrap())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn test_static() {
        let s = Ulid(0x41414141414141414141414141414141).to_string();
        let u = Ulid::from_string(&s).unwrap();
        assert_eq!(&s, "21850M2GA1850M2GA1850M2GA1");
        assert_eq!(u.0, 0x41414141414141414141414141414141);
    }

    #[test]
    fn test_increment() {
        let ulid = Ulid::from_string("01BX5ZZKBKAZZZZZZZZZZZZZZZ").unwrap();
        let ulid = ulid.increment().unwrap();
        assert_eq!("01BX5ZZKBKB000000000000000", ulid.to_string());

        let ulid = Ulid::from_string("01BX5ZZKBKZZZZZZZZZZZZZZZX").unwrap();
        let ulid = ulid.increment().unwrap();
        assert_eq!("01BX5ZZKBKZZZZZZZZZZZZZZZY", ulid.to_string());
        let ulid = ulid.increment().unwrap();
        assert_eq!("01BX5ZZKBKZZZZZZZZZZZZZZZZ", ulid.to_string());
        assert!(ulid.increment().is_none());
    }

    #[test]
    fn test_increment_overflow() {
        let ulid = Ulid(u128::max_value());
        assert!(ulid.increment().is_none());
    }

    #[test]
    fn can_into_thing() {
        let ulid = Ulid::from_str("01FKMG6GAG0PJANMWFN84TNXCD").unwrap();
        let s: String = ulid.into();
        let u: u128 = ulid.into();
        let uu: (u64, u64) = ulid.into();

        assert_eq!(Ulid::from_str(&s).unwrap(), ulid);
        assert_eq!(Ulid::from(u), ulid);
        assert_eq!(Ulid::from(uu), ulid);
    }

    #[test]
    fn default_is_nil() {
        assert_eq!(Ulid::default(), Ulid::nil());
    }

    #[test]
    fn can_display_things() {
        println!("{}", Ulid::nil());
        println!("{}", EncodeError::BufferTooSmall);
        println!("{}", DecodeError::InvalidLength);
        println!("{}", DecodeError::InvalidChar);
    }
}
