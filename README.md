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
let ulid = Ulid::generate();

// Generate a string for a ulid
let s = ulid.to_string();

// Create from a String
let res = Ulid::from_string(&s);

assert_eq!(ulid, res.unwrap());
```

## Crate Features

* **`std` (default)**: Flag to toggle use of `std` and `rand`. Disable this flag for `#[no_std]` support.
* **`serde`**: Enables serialization and deserialization of `Ulid` types via `serde`. ULIDs are serialized using their canonical 26-character representation as defined in the ULID standard. An optional `ulid_as_u128` module is provided, which enables serialization through an `Ulid`'s inner `u128` primitive type. See the [documentation][serde_mod] and [serde docs][serde_docs] for more information.
* **`uuid`**: Implements infallible conversions between ULIDs and UUIDs from the [`uuid`][uuid] crate via the [`std::convert::From`][trait_from] trait.

[serde_mod]: https://docs.rs/ulid/latest/ulid/serde/index.html
[serde_docs]: https://serde.rs/field-attrs.html#with
[uuid]: https://github.com/uuid-rs/uuid
[trait_from]: https://doc.rust-lang.org/std/convert/trait.From.html

## Benchmark

Benchmarks were run on my desktop (Win 10/WSL2 Ubuntu; Ryzen 7 5950x). Run them yourself with `cargo bench`.

```text
test bench_from_string        ... bench:          13 ns/iter (+/- 0)
test bench_from_time          ... bench:          13 ns/iter (+/- 0)
test bench_generator_generate ... bench:          29 ns/iter (+/- 0)
test bench_new                ... bench:          31 ns/iter (+/- 1)
test bench_to_str             ... bench:           7 ns/iter (+/- 0)
test bench_to_string          ... bench:          19 ns/iter (+/- 0)
```

## Changelog

### v3.0.0
* Rename `Ulid::gen()` (previously `Ulid::new()`) to `Ulid::generate()` for better compatability with edition `2024`

### v2.0.1
* Add support for `rand = "0.10"`
* Replaced `MonotonicError::Overflow` with `Overflow`. Generator results can now be properly overflowed using `Overflow::commit_overflow_random()` or `Overflow::commit_overflow_random()`.
    * To keep existing client behavior, drop the `Overflow` value. `generator.generate().map_err(|_| MyOverflowError)`
* Rename `Ulid::new()` to `Ulid::new()` for better compatability with edition `2024`
* Add `Ulid::max()` and `Ulid::is_max()`
* Add `const` to `Ulid::array_to_str()` (Thanks Hermholtz)
* Removed deprecated `Ulid::to_str()`
