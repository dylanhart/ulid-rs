use core::fmt;

/// Length of a string-encoded Ulid
pub const ULID_LEN: usize = 26;

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

const NO_VALUE: u8 = 255;
const LOOKUP: [u8; 256] = [
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 255, 255, 255,
    255, 255, 255, 255, 10, 11, 12, 13, 14, 15, 16, 17, 255, 18, 19, 255, 20, 21, 255, 22, 23, 24,
    25, 26, 255, 27, 28, 29, 30, 31, 255, 255, 255, 255, 255, 255, 10, 11, 12, 13, 14, 15, 16, 17,
    255, 18, 19, 255, 20, 21, 255, 22, 23, 24, 25, 26, 255, 27, 28, 29, 30, 31, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
];

/// Generator code for `LOOKUP`
#[cfg(test)]
#[test]
fn test_lookup_table() {
    let mut lookup = [NO_VALUE; 256];
    for (i, &c) in ALPHABET.iter().enumerate() {
        lookup[c as usize] = i as u8;
        if !(c as char).is_numeric() {
            //lowercase
            lookup[(c + 32) as usize] = i as u8;
        }
    }
    assert_eq!(LOOKUP, lookup);
}

/// An error that can occur when encoding a base32 string
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum EncodeError {
    /// The length of the provided buffer is not large enough
    BufferTooSmall,
}

#[cfg(feature = "std")]
impl std::error::Error for EncodeError {}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let text = match *self {
            EncodeError::BufferTooSmall => "buffer too small",
        };
        write!(f, "{}", text)
    }
}

/// Encode a u128 value to a given buffer. The provided buffer should be at least `ULID_LEN` long.
pub fn encode_to(mut value: u128, buffer: &mut [u8]) -> Result<usize, EncodeError> {
    // NOTE: This function can't be made const because mut refs aren't allowed for some reason

    if buffer.len() < ULID_LEN {
        return Err(EncodeError::BufferTooSmall);
    }

    for i in 0..ULID_LEN {
        buffer[ULID_LEN - 1 - i] = ALPHABET[(value & 0x1f) as usize];
        value >>= 5;
    }

    Ok(ULID_LEN)
}

#[cfg(feature = "std")]
pub fn encode(value: u128) -> String {
    let mut buffer: [u8; ULID_LEN] = [0; ULID_LEN];

    encode_to(value, &mut buffer).expect("unexpected encoding error");

    String::from_utf8(buffer.to_vec()).expect("unexpected failure in base32 encode for ulid")
}

/// An error that can occur when decoding a base32 string
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum DecodeError {
    /// The length of the string does not match the expected length
    InvalidLength,
    /// A non-base32 character was found
    InvalidChar,
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let text = match *self {
            DecodeError::InvalidLength => "invalid length",
            DecodeError::InvalidChar => "invalid character",
        };
        write!(f, "{}", text)
    }
}

pub const fn decode(encoded: &str) -> Result<u128, DecodeError> {
    if encoded.len() != ULID_LEN {
        return Err(DecodeError::InvalidLength);
    }

    let mut value: u128 = 0;

    let bytes = encoded.as_bytes();

    // Manual for loop because Range::iter() isn't const
    let mut i = 0;
    while i < ULID_LEN {
        let val = LOOKUP[bytes[i] as usize];
        if val != NO_VALUE {
            value = (value << 5) | val as u128;
        } else {
            return Err(DecodeError::InvalidChar);
        }
        i += 1;
    }

    Ok(value)
}

#[cfg(all(test, feature = "std"))]
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
        assert_eq!(encode(0xffffffffffffffffffffffffffffffff).len(), ULID_LEN);
        assert_eq!(encode(0x0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f).len(), ULID_LEN);
        assert_eq!(encode(0x00000000000000000000000000000000).len(), ULID_LEN);

        assert_eq!(decode(""), Err(DecodeError::InvalidLength));
        assert_eq!(
            decode("2D9RW50MA499CMAGHM6DD42DT"),
            Err(DecodeError::InvalidLength)
        );
        assert_eq!(
            decode("2D9RW50MA499CMAGHM6DD42DTPP"),
            Err(DecodeError::InvalidLength)
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
            Err(DecodeError::InvalidChar)
        );
        assert_eq!(
            decode("2D9RW50LA499CMAGHM6DD42DTP"),
            Err(DecodeError::InvalidChar)
        );
        assert_eq!(
            decode("2D9RW50IA499CMAGHM6DD42DTP"),
            Err(DecodeError::InvalidChar)
        );
    }
}
