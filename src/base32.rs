const ALPHABET: &'static [u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

lazy_static! {
	static ref LOOKUP: [Option<u8>; 255] = {
		let mut lookup = [None; 255];
		for (i, &c) in ALPHABET.iter().enumerate() {
			lookup[c as usize] = Some(i as u8);
			if !(c as char).is_numeric() {
				//lowercase
				lookup[(c+32) as usize] = Some(i as u8);
			}
		}
		lookup
	};
}

/// An encoding error that can occur when decoding a base32 string
#[derive(Debug, PartialEq)]
pub enum EncodingError {
	/// The length of the string does not match the expected length
	InvalidLength,
	/// A non-base32 character was found
	InvalidChar,
}

pub fn encode(mut msb: u64, mut lsb: u64) -> String {
	let mut buffer: [u8; 26] = [ALPHABET[0]; 26];

	for i in 0..12 {
		buffer[25-i] = ALPHABET[(lsb & 0x1f) as usize];
		lsb >>= 5;
	}

	buffer[13] = ALPHABET[(lsb | ((msb & 1) << 4)) as usize];
	msb >>= 1;

	for i in 0..13 {
		buffer[12-i] = ALPHABET[(msb & 0x1f) as usize];
		msb >>= 5;
	}

	return String::from_utf8(buffer.to_vec())
		.expect("unexpected failure in base32 encode for ulid");
}

pub fn decode(encoded: &str) -> Result<(u64, u64), EncodingError> {
	if encoded.len() != 26 {
		return Err(EncodingError::InvalidLength);
	}

	let mut msb: u64 = 0;
	let mut lsb: u64;

	let bytes = encoded.as_bytes();

	for i in 0..13 {
		if let Some(val) = LOOKUP[bytes[i] as usize] {
			msb = (msb << 5) | val as u64;
		} else {
			return Err(EncodingError::InvalidChar);
		}
	}

	if let Some(val) = LOOKUP[bytes[13] as usize] {
		msb = (msb << 1) | ((val >> 4) & 0x1) as u64;
		lsb = (val & 0xf) as u64;
	} else {
		return Err(EncodingError::InvalidChar);
	}

	for i in 0..12 {
		if let Some(val) = LOOKUP[bytes[14 + i] as usize] {
			lsb = (lsb << 5) | val as u64;
		} else {
			return Err(EncodingError::InvalidChar);
		}
	}

	return Ok((msb, lsb));
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_valid() {
		let val = (0x4141414141414141, 0x4141414141414141);
		assert_eq!(decode("21850M2GA1850M2GA1850M2GA1").unwrap(), val);
		assert_eq!(encode(val.0, val.1), "21850M2GA1850M2GA1850M2GA1");

		let val = (0x4d4e385051444a59, 0x454234335a413756);
		let enc = "2D9RW50MA499CMAGHM6DD42DTP";
		let lower = enc.to_lowercase();
		assert_eq!(encode(val.0, val.1), enc);
		assert_eq!(decode(enc).unwrap(), val);
		assert_eq!(decode(&lower).unwrap(), val);
	}

	#[test]
	fn test_length() {
		assert_eq!(encode(0xffffffffffffffff, 0xffffffffffffffff).len(), 26);
		assert_eq!(encode(0x0f0f0f0f0f0f0f0f, 0x0f0f0f0f0f0f0f0f).len(), 26);
		assert_eq!(encode(0x0000000000000000, 0x0000000000000000).len(), 26);

		assert_eq!(decode(""), Err(EncodingError::InvalidLength));
		assert_eq!(decode("2D9RW50MA499CMAGHM6DD42DT"), Err(EncodingError::InvalidLength));
		assert_eq!(decode("2D9RW50MA499CMAGHM6DD42DTPP"), Err(EncodingError::InvalidLength));
	}

	#[test]
	fn test_chars() {
		for ref c in encode(0xffffffffffffffff, 0xffffffffffffffff).bytes() {
			assert!(ALPHABET.contains(c));
		}
		for ref c in encode(0x0f0f0f0f0f0f0f0f, 0x0f0f0f0f0f0f0f0f).bytes() {
			assert!(ALPHABET.contains(c));
		}
		for ref c in encode(0x0000000000000000, 0x0000000000000000).bytes() {
			assert!(ALPHABET.contains(c));
		}

		assert_eq!(decode("2D9RW50[A499CMAGHM6DD42DTP"), Err(EncodingError::InvalidChar));
		assert_eq!(decode("2D9RW50LA499CMAGHM6DD42DTP"), Err(EncodingError::InvalidChar));
		assert_eq!(decode("2D9RW50IA499CMAGHM6DD42DTP"), Err(EncodingError::InvalidChar));
	}
}
