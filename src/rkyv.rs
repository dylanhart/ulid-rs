#[cfg(test)]
mod tests {
    use crate::Ulid;
    use rkyv::{{from_bytes, to_bytes}, rancor::Error};

    #[test]
    fn test_ulid_roundtrip() {
        // Create a ULID
        let id = Ulid::new();
        let id_string = id.to_string();

        // Serialize it
        let bytes = to_bytes::<Error>(&id).unwrap();

        // Deserialize back
        let deserialized: Ulid = from_bytes::<_, Error>(&bytes).unwrap();

        // Verify equality
        assert_eq!(id, deserialized);
        assert_eq!(id_string, deserialized.to_string());
    }
}
