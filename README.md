# ulid-rs

This is a Rust implementation of the [ulid][ulid] project which provides
Universally Unique Lexicographically Sortable Identifiers.

## Quickstart

```rust
// Generate a ulid
let ulid = Ulid::new();

// Create from a String
let ulid = Ulid::from_string("...");

// Generate a string for a ulid
let s = ulid::to_string()
```

[ulid]: https://github.com/alizain/ulid
