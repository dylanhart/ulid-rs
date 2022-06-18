# ulid-rs

![Build Status](https://github.com/dylanhart/ulid-rs/actions/workflows/ci-build.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/ulid.svg)](https://crates.io/crates/ulid)
[![docs.rs](https://docs.rs/ulid/badge.svg)](https://docs.rs/ulid)

This is a Rust implementation of the [ulid][ulid] project which provides
Universally Unique Lexicographically Sortable Identifiers.

[ulid]: https://github.com/ulid/spec

## Quickstart

```rust
use ulid::Ulid;

// Generate a ulid
let ulid = Ulid::new();

// Generate a string for a ulid
let s = ulid.to_string();

// Create from a String
let res = Ulid::from_string(&s);

assert_eq!(ulid, res.unwrap());
```

## Crate Features

* **`std` (default)**: Flag to toggle use of `std`, `rand`, and `time`. Disable this flag for `#[no_std]` support.
* **`serde`**: Enables serialization and deserialization of `Ulid` types via `serde`. ULIDs are serialized using their canonical 26-character representation as defined in the ULID standard. An optional `ulid_as_u128` module is provided, which enables serialization through an `Ulid`'s inner `u128` primitive type. See the [documentation][serde_mod] and [serde docs][serde_docs] for more information.
* **`uuid`**: Implements infallible conversions between ULIDs and UUIDs from the [`uuid`][uuid] crate via the [`std::convert::From`][trait_from] trait.

[serde_mod]: https://docs.rs/ulid/latest/ulid/serde/index.html
[serde_docs]: https://serde.rs/field-attrs.html#with
[uuid]: https://github.com/uuid-rs/uuid
[trait_from]: https://doc.rust-lang.org/std/convert/trait.From.html

## Benchmark

Benchmarks were run on my desktop (Win 10/WSL2 Ubuntu; Ryzen 7 5800x). Run them yourself with `cargo bench`.

```text
test bench_from_string        ... bench:          13 ns/iter (+/- 1)
test bench_from_time          ... bench:           8 ns/iter (+/- 0)
test bench_generator_generate ... bench:          42 ns/iter (+/- 0)
test bench_new                ... bench:          44 ns/iter (+/- 1)
test bench_to_str             ... bench:           7 ns/iter (+/- 0)
test bench_to_string          ... bench:          20 ns/iter (+/- 0)
```
