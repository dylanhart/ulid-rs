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

pub fn encode(bytes: &[u8; 16]) -> String {
	let mut buffer: [u8; 26] = [0; 26];

	for i in 0..26 {
		let byte = (i * 5) / 8;
		let offset = ((i * 5) % 8) as u8;

		// bits are contained within a byte or the very last byte
		if offset <= 8 - 5 || byte == 15 {
			buffer[25-i] = ALPHABET[((bytes[15 - byte] >> offset) & 0x1f) as usize];
		} else {
			buffer[25-i] = ALPHABET[((bytes[15 - byte] >> offset)
				| ((bytes[15 - byte - 1] & ((1<<(offset-3))-1)) << (8-offset))) as usize];
		}
	}

	return String::from_utf8(buffer.to_vec())
		.expect("unexpected failure in base32 encode for ulid");
}

pub fn decode(encoded: &str) -> Result<[u8; 16], EncodingError> {
	if encoded.len() != 26 {
		return Err(EncodingError::InvalidLength);
	}

	let mut buffer: [u8; 16] = [0; 16];

	for (i, c) in encoded.bytes().rev().enumerate() {
		let byte = (i * 5) / 8;
		let offset = ((i * 5) % 8) as u8;

		if let Some(val) = LOOKUP[c as usize] {

			if offset < 8 - 5 || byte == 15 {
				buffer[15 - byte] |= val << offset;
			} else {
				buffer[15 - byte] |= val << offset;
				buffer[15 - byte - 1] |= val >> (8-offset);
			}

		} else {
			println!("{}", c);
			return Err(EncodingError::InvalidChar);
		}
	}

	return Ok(buffer);
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_valid() {
		assert_eq!(decode("21850M2GA1850M2GA1850M2GA1").unwrap(), [b'A'; 16]);
		assert_eq!(encode(&[b'A'; 16]), "21850M2GA1850M2GA1850M2GA1");

		let val = [b'M', b'N', b'8', b'P', b'Q', b'D', b'J', b'Y', b'E', b'B',
			b'4', b'3', b'Z', b'A', b'7', b'V'];
		let enc = "2D9RW50MA499CMAGHM6DD42DTP";
		let lower = enc.to_lowercase();
		assert_eq!(encode(&val), enc);
		assert_eq!(decode(enc).unwrap(), val);
		assert_eq!(decode(&lower).unwrap(), val);
	}

	#[test]
	fn test_length() {
		assert_eq!(encode(&[255; 16]).len(), 26);
		assert_eq!(encode(&[26; 16]).len(), 26);
		assert_eq!(encode(&[0; 16]).len(), 26);

		assert_eq!(decode(""), Err(EncodingError::InvalidLength));
		assert_eq!(decode("2D9RW50MA499CMAGHM6DD42DT"), Err(EncodingError::InvalidLength));
		assert_eq!(decode("2D9RW50MA499CMAGHM6DD42DTPP"), Err(EncodingError::InvalidLength));
	}

	#[test]
	fn test_chars() {
		for ref c in encode(&[255; 16]).bytes() {
			assert!(ALPHABET.contains(c));
		}
		for ref c in encode(&[26; 16]).bytes() {
			assert!(ALPHABET.contains(c));
		}
		for ref c in encode(&[0; 16]).bytes() {
			assert!(ALPHABET.contains(c));
		}

		assert_eq!(decode("2D9RW50[A499CMAGHM6DD42DTP"), Err(EncodingError::InvalidChar));
		assert_eq!(decode("2D9RW50LA499CMAGHM6DD42DTP"), Err(EncodingError::InvalidChar));
		assert_eq!(decode("2D9RW50IA499CMAGHM6DD42DTP"), Err(EncodingError::InvalidChar));
	}
}
