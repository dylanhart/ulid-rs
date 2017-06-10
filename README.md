# ulid-rs

This is a Rust implementation of the [ulid][ulid] project which provides
Universally Unique Lexicographically Sortable Identifiers.

## Quickstart

```rust
// Generate a ulid
let ulid = Ulid::new();

// Generate a string for a ulid
let s = ulid.to_string();

// Create from a String
let res = Ulid::from_string(&s);

assert_eq!(ulid, res.unwrap());
```

[ulid]: https://github.com/alizain/ulid
