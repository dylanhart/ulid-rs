//! Conversions between ULID and UUID.

use crate::Ulid;
use uuid::Uuid;

#[cfg_attr(docsrs, doc(cfg(feature = "uuid")))]
impl From<Uuid> for Ulid {
    fn from(uuid: Uuid) -> Self {
        Ulid(uuid.as_u128())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "uuid")))]
impl From<Ulid> for Uuid {
    fn from(ulid: Ulid) -> Self {
        Uuid::from_u128(ulid.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn uuid_cycle() {
        #[cfg(feature = "std")]
        let ulid = Ulid::new();
        #[cfg(not(feature = "std"))]
        let ulid = Ulid::from_parts(
            0x0000_1020_3040_5060_u64,
            0x0000_0000_0000_0102_0304_0506_0708_090A_u128,
        );

        let uuid: Uuid = ulid.into();
        let ulid2: Ulid = uuid.into();

        assert_eq!(ulid, ulid2);
    }

    #[test]
    fn uuid_str_cycle() {
        let uuid_txt = "771a3bce-02e9-4428-a68e-b1e7e82b7f9f";
        let ulid_txt = "3Q38XWW0Q98GMAD3NHWZM2PZWZ";
        let mut buf = uuid::Uuid::encode_buffer();

        let ulid: Ulid = Uuid::parse_str(uuid_txt).unwrap().into();
        let ulid_str = ulid.to_str(&mut buf).unwrap();
        assert_eq!(ulid_str, ulid_txt);

        #[cfg(feature = "std")]
        assert_eq!(ulid.to_string().as_str(), ulid_txt);

        let uuid: Uuid = ulid.into();
        let uuid_str = uuid.hyphenated().encode_lower(&mut buf);
        assert_eq!(uuid_str, uuid_txt);
    }
}
