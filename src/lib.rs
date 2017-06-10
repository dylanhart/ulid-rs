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

extern crate byteorder;
extern crate chrono;
#[macro_use] extern crate lazy_static;
extern crate rand;

mod base32;

use byteorder::{ByteOrder, BigEndian};
use chrono::prelude::{DateTime, UTC, TimeZone};
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
pub struct Ulid([u8; 16]);

impl Ulid {

	/// Creates a new Ulid with the current time
	pub fn new() -> Ulid {
		Ulid::from_datetime(UTC::now())
	}

	/// Creates a new Ulid with the given datetime
	///
	/// This can be useful when migrating data to use Ulid identifiers
	pub fn from_datetime<T: TimeZone>(datetime: DateTime<T>) -> Ulid {
		let mut buffer: [u8; 16] = [0; 16];

		let timestamp = datetime.timestamp() * 1000
			+ (datetime.timestamp_subsec_millis() as i64);

		let timebits = (timestamp & ((1<<48)-1)) as u64;

		BigEndian::write_u16(&mut buffer[..2], ((timebits >> 32) & ((1<<16)-1)) as u16);
		BigEndian::write_u32(&mut buffer[2..6], (timebits & ((1<<32)-1)) as u32);
		buffer[6..].copy_from_slice(&rand::random::<[u8; 10]>());

		return Ulid(buffer);
	}

	/// Creates a Ulid from a Crockford Base32 encoded string
	///
	/// An EncodingError will be returned when the given string is not formated
	/// properly.
	pub fn from_string(encoded: &str) -> Result<Ulid, EncodingError> {
		return base32::decode(encoded).map(Ulid);
	}

	/// Gets the datetime of when this Ulid was created accurate to 1ms
	pub fn datetime(&self) -> DateTime<UTC> {
		let stamp = BigEndian::read_u64(&self.0[..8]) >> 16;
		let secs = stamp / 1000;
		let millis = stamp % 1000;
		return UTC.timestamp(secs as i64, (millis*1000000) as u32);
	}

	/// Creates a Crockford Base32 encoded string that represents this Ulid
	pub fn to_string(&self) -> String {
		return base32::encode(&self.0);
	}

}

impl <'a> Into<String> for &'a Ulid {
	fn into(self) -> String {
		self.to_string()
	}
}

impl <'a> Into<(u64, u64)> for &'a Ulid {
	fn into(self) -> (u64, u64) {
		(
			BigEndian::read_u64(&self.0[..8]),
			BigEndian::read_u64(&self.0[8..]),
		)
	}
}

impl From<(u64, u64)> for Ulid {
	fn from(tuple: (u64, u64)) -> Ulid {
		let mut buffer = [0; 16];
		BigEndian::write_u64(&mut buffer[..8], tuple.0);
		BigEndian::write_u64(&mut buffer[8..], tuple.1);
		return Ulid(buffer);
	}
}

#[cfg(test)]
mod tests {
	use super::Ulid;
	use chrono::prelude::*;
	use chrono::Duration;

	#[test]
	fn test_dynamic() {
		let ulid = Ulid::new();
		let encoded = ulid.to_string();
		let ulid2 = Ulid::from_string(&encoded)
			.expect("failed to deserialize");

		println!("{}", encoded);
		println!("{:?}", ulid);
		println!("{:?}", ulid2);
		assert_eq!(ulid, ulid2);
	}

	#[test]
	fn test_static() {
		let s = Ulid([b'A'; 16]).to_string();
		let u = Ulid::from_string(&s).unwrap();
		assert_eq!(&s, "21850M2GA1850M2GA1850M2GA1");
		assert_eq!(u.0, [b'A'; 16]);
	}

	#[test]
	fn test_order() {
		let dt = UTC::now();
		let ulid1 = Ulid::from_datetime(dt);
		let ulid2 = Ulid::from_datetime(dt + Duration::milliseconds(1));
		assert!(ulid1 < ulid2);
	}

	#[test]
	fn test_conversions() {
		let val = [b'M', b'N', b'8', b'P', b'Q', b'D', b'J', b'Y', b'E', b'B',
			b'4', b'3', b'Z', b'A', b'7', b'V'];
		let ulid = Ulid(val);

		assert_eq!(Into::<(u64, u64)>::into(&ulid), (5570451706715851353, 4990608732242130774));
		assert_eq!(Ulid::from((5570451706715851353, 4990608732242130774)), ulid);

		let dt = UTC::now();
		let ulid = Ulid::from_datetime(dt);

		println!("{:?}, {:?}", dt, ulid.datetime());
		assert!(ulid.datetime() <= dt);
		assert!(ulid.datetime() + Duration::milliseconds(1) >= dt);
	}
}
