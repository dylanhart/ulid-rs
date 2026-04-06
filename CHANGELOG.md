# Changelog

## 2.0.0

### Changed

- Bumped `rand` dependency to 0.10. The trait bound on `with_source`, `from_datetime_with_source`, `generate_with_source`, and `generate_from_datetime_with_source` changed from `rand::Rng` to `rand::RngExt` due to upstream renaming in rand 0.10.
