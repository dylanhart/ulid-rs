//! Serialization and deserialization.
//!
//! By default, serialization and deserialization go through ULID's 26-character
//! canonical string representation as set by the ULID standard.
//!
//! ULIDs can optionally be serialized as u128 integers using the `ulid_as_u128`
//! module. See the module's documentation for examples.

use crate::{Ulid, ULID_LEN};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Ulid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buffer = [0; ULID_LEN];
        let text = self.to_str(&mut buffer).unwrap();
        text.serialize(serializer)
    }
}

struct UlidVisitor(&'static str);
impl<'de> serde::de::Visitor<'de> for UlidVisitor {
    type Value = Ulid;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str(self.0)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        #[cfg(feature = "uuid")]
        if matches!(value.len(), 32 | 36 | 38 | 45) {
            return match uuid::Uuid::try_parse(value) {
                Ok(a) => Ok(Ulid::from(a)),
                Err(e) => Err(serde::de::Error::custom(e)),
            };
        }
        Ulid::from_string(value).map_err(serde::de::Error::custom)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        // allow either the 16 bytes as is, or a wrongly typed str
        match v.len() {
            16 => {
                let ptr = v.as_ptr() as *const [u8; 16];
                Ok(Ulid::from_bytes(*unsafe { &*ptr }))
            }
            crate::ULID_LEN => Ulid::from_string(unsafe { core::str::from_utf8_unchecked(v) })
                .map_err(serde::de::Error::custom),
            #[cfg(feature = "uuid")]
            32 | 36 | 38 | 45 => match uuid::Uuid::try_parse_ascii(v) {
                Ok(a) => Ok(Ulid::from(a)),
                Err(e) => Err(serde::de::Error::custom(e)),
            },
            len => Err(E::invalid_length(len, &self.0)),
        }
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Ulid(v))
    }
}

impl<'de> Deserialize<'de> for Ulid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(UlidVisitor("an ulid string or value"))
    }
}

/// Serialization and deserialization of ULIDs through their inner u128 type.
///
/// To use it, annotate a field with
/// `#[serde(with = "ulid_as_u128")]`,
/// `#[serde(serialize_with = "ulid_as_u128")]`, or
/// `#[serde(deserialize_with = "ulid_as_u128")]`.
///
/// # Examples
/// ```
/// # use ulid::Ulid;
/// # use ulid::serde::ulid_as_u128;
/// # use serde::{Serialize, Deserialize};
/// #[derive(Serialize, Deserialize)]
/// struct U128Example {
///     #[serde(with = "ulid_as_u128")]
///     identifier: Ulid
/// }
/// ```
pub mod ulid_as_u128 {
    use crate::Ulid;
    use serde::{Deserializer, Serialize, Serializer};

    /// Serializes a ULID as a u128 type.
    pub fn serialize<S>(value: &Ulid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.0.serialize(serializer)
    }

    /// Deserializes a ULID from a u128 type.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Ulid, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u128(super::UlidVisitor("an ulid value as u128"))
    }
}

/// Serialization and deserialization of ULIDs through UUID strings.
///
/// To use this module, annotate a field with
/// `#[serde(with = "ulid_as_uuid")]`,
/// `#[serde(serialize_with = "ulid_as_uuid")]`, or
/// `#[serde(deserialize_with = "ulid_as_uuid")]`.
///
/// # Examples
/// ```
/// # use ulid::Ulid;
/// # use ulid::serde::ulid_as_uuid;
/// # use serde::{Serialize, Deserialize};
/// #[derive(Serialize, Deserialize)]
/// struct UuidExample {
///     #[serde(with = "ulid_as_uuid")]
///     identifier: Ulid
/// }
/// ```
#[cfg(all(feature = "uuid", feature = "serde"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "uuid", feature = "serde"))))]
pub mod ulid_as_uuid {
    use crate::Ulid;
    use serde::{Deserializer, Serializer};
    use uuid::Uuid;

    /// Converts the ULID to a UUID and serializes it as a string.
    pub fn serialize<S>(value: &Ulid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let uuid: Uuid = (*value).into();
        let mut buffer = uuid::Uuid::encode_buffer();
        let form = uuid.as_hyphenated().encode_lower(&mut buffer);
        serializer.serialize_str(form)
    }

    /// Deserializes a ULID from a string containing a UUID.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Ulid, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(super::UlidVisitor("an uuid string"))
    }
}
