//! Implemements encoding/decoding for Mulsi

use musli::{
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
