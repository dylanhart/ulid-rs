//! Implemements encoding/decoding for Mulsi

use musli_core::{
    mode::{Binary, Text},
    Allocator, Context, Decode, Decoder, Encode, Encoder,
};

use crate::Ulid;

impl Encode<Binary> for Ulid {
    type Encode = Self;

    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_ENCODE: bool = false;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.encode_array(&self.to_bytes())
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl Encode<Text> for Ulid {
    type Encode = Self;

    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_ENCODE: bool = false;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.collect_string(self)
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, A> Decode<'de, Binary, A> for Ulid
where
    A: Allocator,
{
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_array::<16>().map(Ulid::from)
    }
}

impl<'de, A> Decode<'de, Text, A> for Ulid
where
    A: Allocator,
{
    // Not packed since it doesn't have a strongly defined memory layout, even
    // though it has a particular size.
    const IS_BITWISE_DECODE: bool = false;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let cx = decoder.cx();
        decoder.decode_unsized(|string: &str| Ulid::from_string(string).map_err(cx.map()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use musli::json::Encoding;
    use musli::mode::*;

    #[test]
    fn test_roundtrip() {
        let id = Ulid::new();
        let id_string = id.to_string();

        let bin = musli::wire::to_vec(&id).unwrap();
        let deserialized = musli::wire::from_slice(&bin).unwrap();

        assert_eq!(id, deserialized);
        assert_eq!(id_string, deserialized.to_string());
    }

    #[test]
    fn test_roundtrip_modes() {
        const TEXT: Encoding<Text> = Encoding::new();
        const BINARY: Encoding<Binary> = Encoding::new().with_mode();

        let id = Ulid::from_string("01KE1SGJNSGM7AMZNDWQ24B1PT").unwrap();

        let named = TEXT.to_vec(&id).unwrap();
        assert_eq!(named.as_slice(), br#""01KE1SGJNSGM7AMZNDWQ24B1PT""#);

        let indexed = BINARY.to_vec(&id).unwrap();
        assert_eq!(
            indexed.as_slice(),
            br#"[1,155,131,152,74,185,133,14,170,126,173,229,196,69,134,218]"#
        );
    }
}
