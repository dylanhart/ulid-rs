#![warn(missing_docs)]
//! # ulid-rs
//!
//! This is a Rust implementation of the [ulid][ulid] project which provides
//! Universally Unique Lexicographically Sortable Identifiers.
//!
//! [ulid]: https://github.com/alizain/ulid
//!
//!
//! ## Quickstart
//!
//! ```rust
//! # use ulid::Ulid;
//! // Generate a ulid
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

extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate rand;

mod base32;

use std::str::FromStr;
use chrono::prelude::{DateTime, TimeZone, Utc};
use std::fmt;

pub use base32::EncodingError;

/// A Ulid is a unique 128-bit lexicographically sortable identifier
///
/// Canonically, it is represented as a 26 character Crockford Base32 encoded
/// string.
///
/// Of the 128-bits, the first 48 are a unix timestamp in milliseconds. The
/// remaining 80 are random. The first 48 provide for lexicographic sorting and
/// the remaining 80 ensure that the identifier is unique.
#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
pub struct Ulid(pub u128);

impl Ulid {
    /// Creates a new Ulid with the current time
    pub fn new() -> Ulid {
        Ulid::from_datetime(Utc::now())
    }

    /// Creates a new Ulid using data from the given random number generator
    pub fn with_source<R: rand::Rng>(source: &mut R) -> Ulid {
        Ulid::from_datetime_with_source(Utc::now(), source)
    }

    /// Creates a new Ulid with the given datetime
    ///
    /// This can be useful when migrating data to use Ulid identifiers
    pub fn from_datetime<T: TimeZone>(datetime: DateTime<T>) -> Ulid {
        Ulid::from_datetime_with_source(datetime, &mut rand::thread_rng())
    }

    /// Creates a new Ulid with the given datetime and random number generator
    pub fn from_datetime_with_source<T, R>(datetime: DateTime<T>, source: &mut R) -> Ulid
    where
        T: TimeZone,
        R: rand::Rng,
    {
        let timestamp = datetime.timestamp_millis();
        let timebits = (timestamp & ((1 << 48) - 1)) as u64;

        let msb = timebits << 16 | u64::from(source.gen::<u16>());
        let lsb = source.gen::<u64>();
        Ulid::from((msb, lsb))
    }

    /// Creates a Ulid from a Crockford Base32 encoded string
    ///
    /// An EncodingError will be returned when the given string is not formated
    /// properly.
    pub fn from_string(encoded: &str) -> Result<Ulid, EncodingError> {
        base32::decode(encoded).map(Ulid)
    }

    /// The 'nil Ulid'.
    ///
    /// The nil Ulid is special form of Ulid that is specified to have
    /// all 128 bits set to zero.
    ///
    /// # Examples
    ///
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
    pub fn nil() -> Ulid {
        Ulid(0)
    }

    /// Gets the datetime of when this Ulid was created accurate to 1ms
    pub fn datetime(&self) -> DateTime<Utc> {
        let stamp = self.timestamp_ms();
        let secs = stamp / 1000;
        let millis = stamp % 1000;
        Utc.timestamp(secs as i64, (millis * 1_000_000) as u32)
    }

    /// Gets the timestamp section of this ulid
    pub fn timestamp_ms(&self) -> u64 {
        (self.0 >> 80) as u64
    }

    /// Creates a Crockford Base32 encoded string that represents this Ulid
    pub fn to_string(&self) -> String {
        base32::encode(self.0)
    }

    /// Test if the Ulid is nil
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ulid::Ulid;
    ///
    /// let nil = Ulid::nil();
    ///
    /// assert!(nil.is_nil());
    /// ```
    pub fn is_nil(&self) -> bool {
        self.0 == 0u128
    }

    /// increment the random number, make sure that the ts millis stays the same
    fn increment(&self) -> Option<Ulid> {
        let prev_ts_ms = self.timestamp_ms();
        let next = Ulid(self.0 + 1);
        if prev_ts_ms == next.timestamp_ms() {
            Some(next)
        } else {
            None
        }
    }
}

impl Default for Ulid {
    fn default() -> Self {
        Ulid::nil()
    }
}

impl<'a> Into<String> for &'a Ulid {
    fn into(self) -> String {
        self.to_string()
    }
}

impl From<(u64, u64)> for Ulid {
    fn from((msb, lsb): (u64, u64)) -> Self {
        Ulid(u128::from(msb) << 64 | u128::from(lsb))
    }
}

impl Into<(u64, u64)> for Ulid {
    fn into(self) -> (u64, u64) {
        (
            (self.0 >> 64) as u64,
            (self.0 & 0xffff_ffff_ffff_ffff) as u64,
        )
    }
}

impl From<u128> for Ulid {
    fn from(value: u128) -> Ulid {
        Ulid(value)
    }
}

impl Into<u128> for Ulid {
    fn into(self) -> u128 {
        self.0
    }
}

impl FromStr for Ulid {
    type Err = EncodingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ulid::from_string(s)
    }
}

impl fmt::Display for Ulid {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.to_string())
    }
}

/// generator for monotonic ulids
pub struct Generator {
    previous: Option<Ulid>
}

/// error while trying to generate a monotonic increment in the same millisecond
#[derive(Debug, PartialEq)]
pub enum MonotonicError {
    /// would overflow into the next millisecond
    Overflow,
}

impl Generator {
    /// create a new ulid generator for monotonically ordered ulids
    pub fn new() -> Generator { Generator { previous: None } }

    /// use Utc::now to generate a monotonically ordered ulid
    pub fn generate(&mut self) -> Result<Ulid, MonotonicError> {
        let now = Utc::now();
        self.generate_from_datetime(now)
    }

    /// generate a new monotonically ordered ulid. will use previous calls timestamp to
    /// compare and make sure that each subsequent call is monotonically increasing
    ///
    /// # Example
    /// ```rust
    /// let dt = Utc::now();
    /// let mut gen = Generator::new();
    /// let ulid1 = gen.generate_from_datetime(dt).unwrap();
    /// let ulid2 = gen.generate_from_datetime(dt).unwrap();
    ///
    /// assert!(ulid1 < ulid2);
    /// ```
    pub fn generate_from_datetime<T: TimeZone>(&mut self, datetime: DateTime<T>) -> Result<Ulid, MonotonicError> {
        if let Some(prev) = self.previous {
            let last_ms = prev.timestamp_ms() as i64;
            // maybe time went backward, or it is the same ms.
            // increment instead of generating a new random so that it is monotonic
            if datetime.timestamp_millis() <= last_ms {
                if let Some(next) = prev.increment() {
                    self.previous = Some(next);
                    return Ok(next.clone())
                } else {
                    return Err(MonotonicError::Overflow)
                }
            }
        }
        let next = Ulid::from_datetime(datetime);
        self.previous = Some(next);
        Ok(next.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::Ulid;
    use chrono::prelude::*;
    use chrono::Duration;
    use super::Generator;

    #[test]
    fn test_dynamic() {
        let ulid = Ulid::new();
        let encoded = ulid.to_string();
        let ulid2 = Ulid::from_string(&encoded).expect("failed to deserialize");

        println!("{}", encoded);
        println!("{:?}", ulid);
        println!("{:?}", ulid2);
        assert_eq!(ulid, ulid2);
    }

    #[test]
    fn test_static() {
        let s = Ulid(0x41414141414141414141414141414141).to_string();
        let u = Ulid::from_string(&s).unwrap();
        assert_eq!(&s, "21850M2GA1850M2GA1850M2GA1");
        assert_eq!(u.0, 0x41414141414141414141414141414141);
    }

    #[test]
    fn test_source() {
        use rand::rngs::OsRng;
        let mut source = OsRng::new().expect("could not create OS Rng");

        let u1 = Ulid::with_source(&mut source);
        let dt = Utc::now() + Duration::milliseconds(1);
        let u2 = Ulid::from_datetime_with_source(dt, &mut source);
        let u3 = Ulid::from_datetime_with_source(dt, &mut source);

        assert!(u1 < u2);
        assert!(u2 != u3);
    }

    #[test]
    fn test_order() {
        let dt = Utc::now();
        let ulid1 = Ulid::from_datetime(dt);
        let ulid2 = Ulid::from_datetime(dt + Duration::milliseconds(1));
        assert!(ulid1 < ulid2);
    }

    #[test]
    fn test_order_monotonic() {
        let dt = Utc::now();
        let mut gen = Generator::new();
        let ulid1 = gen.generate_from_datetime(dt).unwrap();
        let ulid2 = gen.generate_from_datetime(dt).unwrap();
        assert!(ulid1 < ulid2);
    }

    #[test]
    fn test_increment() {
        let ulid = Ulid::from_string("01BX5ZZKBKZZZZZZZZZZZZZZZX").unwrap();
        let ulid = ulid.increment().unwrap(); // 01BX5ZZKBKZZZZZZZZZZZZZZZY
        let ulid = ulid.increment().unwrap(); // 01BX5ZZKBKZZZZZZZZZZZZZZZZ
        assert!(ulid.increment().is_none());
    }

    #[test]
    fn test_datetime() {
        let dt = Utc::now();
        let ulid = Ulid::from_datetime(dt);

        println!("{:?}, {:?}", dt, ulid.datetime());
        assert!(ulid.datetime() <= dt);
        assert!(ulid.datetime() + Duration::milliseconds(1) >= dt);
    }

    #[test]
    fn test_timestamp() {
        let dt = Utc::now();
        let ulid = Ulid::from_datetime(dt);
        let ts = dt.timestamp() as u64 * 1000 + dt.timestamp_subsec_millis() as u64;

        assert_eq!(ulid.timestamp_ms(), ts);
    }
}
