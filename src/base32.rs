use std::fmt;
use lazy_static::lazy_static;

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

lazy_static! {
    static ref LOOKUP: [Option<u8>; 256] = {
        let mut lookup = [None; 256];
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
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum EncodingError {
    /// The length of the string does not match the expected length
    InvalidLength,
    /// A non-base32 character was found
    InvalidChar,
}

impl fmt::Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let text = match *self {
            EncodingError::InvalidLength => "invalid length",
            EncodingError::InvalidChar => "invalid character",
        };
        write!(f, "{}", text)
    }
}

pub fn encode(mut value: u128) -> String {
    let mut buffer: [u8; 26] = [ALPHABET[0]; 26];

    for i in 0..26 {
        buffer[25 - i] = ALPHABET[(value & 0x1f) as usize];
        value >>= 5;
    }

    String::from_utf8(buffer.to_vec()).expect("unexpected failure in base32 encode for ulid")
}

pub fn decode(encoded: &str) -> Result<u128, EncodingError> {
    if encoded.len() != 26 {
        return Err(EncodingError::InvalidLength);
    }

    let mut value: u128 = 0;

    let bytes = encoded.as_bytes();

    for i in 0..26 {
        if let Some(val) = LOOKUP[bytes[i] as usize] {
            value = (value << 5) | u128::from(val);
        } else {
            return Err(EncodingError::InvalidChar);
        }
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid() {
        let val = 0x41414141414141414141414141414141;
        assert_eq!(decode("21850M2GA1850M2GA1850M2GA1").unwrap(), val);
        assert_eq!(encode(val), "21850M2GA1850M2GA1850M2GA1");

        let val = 0x4d4e385051444a59454234335a413756;
        let enc = "2D9RW50MA499CMAGHM6DD42DTP";
        let lower = enc.to_lowercase();
        assert_eq!(encode(val), enc);
        assert_eq!(decode(enc).unwrap(), val);
        assert_eq!(decode(&lower).unwrap(), val);
    }

    #[test]
    fn test_length() {
        assert_eq!(encode(0xffffffffffffffffffffffffffffffff).len(), 26);
        assert_eq!(encode(0x0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f).len(), 26);
        assert_eq!(encode(0x00000000000000000000000000000000).len(), 26);

        assert_eq!(decode(""), Err(EncodingError::InvalidLength));
        assert_eq!(
            decode("2D9RW50MA499CMAGHM6DD42DT"),
            Err(EncodingError::InvalidLength)
        );
        assert_eq!(
            decode("2D9RW50MA499CMAGHM6DD42DTPP"),
            Err(EncodingError::InvalidLength)
        );
    }

    #[test]
    fn test_chars() {
        for ref c in encode(0xffffffffffffffffffffffffffffffff).bytes() {
            assert!(ALPHABET.contains(c));
        }
        for ref c in encode(0x0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f).bytes() {
            assert!(ALPHABET.contains(c));
        }
        for ref c in encode(0x00000000000000000000000000000000).bytes() {
            assert!(ALPHABET.contains(c));
        }

        assert_eq!(
            decode("2D9RW50[A499CMAGHM6DD42DTP"),
            Err(EncodingError::InvalidChar)
        );
        assert_eq!(
            decode("2D9RW50LA499CMAGHM6DD42DTP"),
            Err(EncodingError::InvalidChar)
        );
        assert_eq!(
            decode("2D9RW50IA499CMAGHM6DD42DTP"),
            Err(EncodingError::InvalidChar)
        );
    }
}
