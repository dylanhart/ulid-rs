//! Conversions between ULID and UUID.

use crate::Ulid;
use uuid::Uuid;

impl From<Uuid> for Ulid {
    fn from(uuid: Uuid) -> Self {
        Ulid(uuid.as_u128())
    }
}
impl From<Ulid> for Uuid {
    fn from(ulid: Ulid) -> Self {
        Uuid::from_u128(ulid.0)
    }
}
