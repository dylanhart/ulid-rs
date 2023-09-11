use crate::Ulid;
use bytes::BufMut;
use bytes::BytesMut;
use postgres_types::accepts;
use postgres_types::to_sql_checked;
use postgres_types::{FromSql, IsNull, ToSql, Type};
use std::error::Error;
use std::u128;

impl FromSql<'_> for Ulid {
    fn from_sql(_ty: &Type, raw: &[u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        if raw.len() != 16 {
            return Err("invalid message length: uuid size mismatch".into());
        }
        let mut bytes = [0; 16];
        bytes.copy_from_slice(raw);
        Ok(Ulid(u128::from_be_bytes(bytes)))
    }
    accepts!(UUID);
}

impl ToSql for Ulid {
    fn to_sql(&self, _: &Type, w: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        let bytes: u128 = self.0.into();
        w.put_slice(&bytes.to_be_bytes());
        Ok(IsNull::No)
    }

    accepts!(UUID);
    to_sql_checked!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ulid;
    use postgres_types::{FromSql, Type};
    use std::io::Read;

    #[test]
    fn postgres_cycle() {
        let ulid = Ulid::from_string("3Q38XWW0Q98GMAD3NHWZM2PZWZ").unwrap();

        let mut w = bytes::BytesMut::new();
        let t = &Type::UUID;
        let _ = ulid.to_sql(t, &mut w);

        assert_eq!(16, w.len());

        let bs = w.bytes().map(|v| v.unwrap()).collect::<Vec<u8>>();

        assert_eq!(ulid, Ulid::from_sql(t, &bs).unwrap());
    }
}
