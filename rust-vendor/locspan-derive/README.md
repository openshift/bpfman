# Derive macros for the `locspan` code mapping library.

[![CI](https://github.com/timothee-haudebourg/locspan-derive/workflows/CI/badge.svg)](https://github.com/timothee-haudebourg/locspan/actions)
[![Crate informations](https://img.shields.io/crates/v/locspan-derive.svg?style=flat-square)](https://crates.io/crates/locspan)
[![License](https://img.shields.io/crates/l/locspan-derive.svg?style=flat-square)](https://github.com/timothee-haudebourg/locspan-derive#license)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/locspan)

This library provides the `StrippedPartialEq` derive macro
used to automatically implement the `StrippedPartialEq` comparison
trait defined in the `locspan` library.

### Usage

```rust
use locspan::Loc;
use locspan_derive::StrippedPartialEq;

// Implement `StrippedPartialEq` for the `Foo` type.
// Type parameters will be required to implement
// `StrippedPartialEq` themselves unless they are marked
// with `#[stripped]`.
#[derive(StrippedPartialEq)]
struct Foo<T, #[stripped] S, #[stripped] P> {
  a: Loc<T, S, P>,

  // Files are compared using `StrippedPartialEq`
  // unless they are marked with `#[stripped]`, in
  // which case `PartialEq` is used.
  #[stripped]
  b: std::path::PathBuf
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
