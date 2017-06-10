# ulid-rs

[![Build Status](https://travis-ci.org/dylanhart/ulid-rs.svg?branch=master)](https://travis-ci.org/dylanhart/ulid-rs)
[![Crates.io](https://img.shields.io/crates/v/ulid.svg)](https://crates.io/crates/ulid)

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
