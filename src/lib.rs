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
//!
//! assert_eq!(ulid, res.unwrap());
//! ```

extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate rand;

mod base32;

use chrono::prelude::{DateTime, TimeZone, UTC};
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
#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub struct Ulid(pub u64, pub u64);

impl Ulid {
    /// Creates a new Ulid with the current time
    pub fn new() -> Ulid {
        Ulid::from_datetime(UTC::now())
    }

    /// Creates a new Ulid using data from the given random number generator
    pub fn with_source<R: rand::Rng>(source: &mut R) -> Ulid {
        Ulid::from_datetime_with_source(UTC::now(), source)
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
        let timestamp = datetime.timestamp() * 1000 + (datetime.timestamp_subsec_millis() as i64);

        let timebits = (timestamp & ((1 << 48) - 1)) as u64;

        return Ulid(
            timebits << 16 | source.gen::<u16>() as u64,
            source.gen::<u64>(),
        );
    }

    /// Creates a Ulid from a Crockford Base32 encoded string
    ///
    /// An EncodingError will be returned when the given string is not formated
    /// properly.
    pub fn from_string(encoded: &str) -> Result<Ulid, EncodingError> {
        return base32::decode(encoded).map(|(msb, lsb)| Ulid(msb, lsb));
    }

    /// Gets the datetime of when this Ulid was created accurate to 1ms
    pub fn datetime(&self) -> DateTime<UTC> {
        let stamp = self.0 >> 16;
        let secs = stamp / 1000;
        let millis = stamp % 1000;
        return UTC.timestamp(secs as i64, (millis * 1000000) as u32);
    }

    /// Gets the timestamp section of this ulid
    pub fn timestamp_ms(&self) -> u64 {
        return self.0 >> 16;
    }

    /// Creates a Crockford Base32 encoded string that represents this Ulid
    pub fn to_string(&self) -> String {
        return base32::encode(self.0, self.1);
    }
}

impl<'a> Into<String> for &'a Ulid {
    fn into(self) -> String {
        self.to_string()
    }
}

impl From<(u64, u64)> for Ulid {
    fn from(tuple: (u64, u64)) -> Self {
        Ulid(tuple.0, tuple.1)
    }
}

impl Into<(u64, u64)> for Ulid {
    fn into(self) -> (u64, u64) {
        (self.0, self.1)
    }
}

impl fmt::Display for Ulid {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::Ulid;
    use chrono::prelude::*;
    use chrono::Duration;
    use rand;

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
        let s = Ulid(0x4141414141414141, 0x4141414141414141).to_string();
        let u = Ulid::from_string(&s).unwrap();
        assert_eq!(&s, "21850M2GA1850M2GA1850M2GA1");
        assert_eq!(u.0, 0x4141414141414141);
        assert_eq!(u.1, 0x4141414141414141);
    }

    #[test]
    fn test_source() {
        let mut source = rand::os::OsRng::new().expect("could not create OS Rng");

        let u1 = Ulid::with_source(&mut source);
        let dt = UTC::now() + Duration::milliseconds(1);
        let u2 = Ulid::from_datetime_with_source(dt, &mut source);
        let u3 = Ulid::from_datetime_with_source(dt, &mut source);

        assert!(u1 < u2);
        assert!(u2 != u3);
    }

    #[test]
    fn test_order() {
        let dt = UTC::now();
        let ulid1 = Ulid::from_datetime(dt);
        let ulid2 = Ulid::from_datetime(dt + Duration::milliseconds(1));
        assert!(ulid1 < ulid2);
    }

    #[test]
    fn test_datetime() {
        let dt = UTC::now();
        let ulid = Ulid::from_datetime(dt);

        println!("{:?}, {:?}", dt, ulid.datetime());
        assert!(ulid.datetime() <= dt);
        assert!(ulid.datetime() + Duration::milliseconds(1) >= dt);
    }

    #[test]
    fn test_timestamp() {
        let dt = UTC::now();
        let ulid = Ulid::from_datetime(dt);
        let ts = dt.timestamp() as u64 * 1000 + dt.timestamp_subsec_millis() as u64;

        assert_eq!(ulid.timestamp_ms(), ts);
    }
}
