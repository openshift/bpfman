# Lexical JSON number types

[![CI](https://github.com/timothee-haudebourg/json-number/workflows/CI/badge.svg)](https://github.com/timothee-haudebourg/json-number/actions)
[![Crate informations](https://img.shields.io/crates/v/json-number.svg?style=flat-square)](https://crates.io/crates/json-number)
[![License](https://img.shields.io/crates/l/json-number.svg?style=flat-square)](https://github.com/timothee-haudebourg/json-number#license)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/json-number)

This is a simple library for parsing and storing JSON numbers according
to the [JSON specification](https://www.json.org/json-en.html).
It provides two types, the unsized `Number` type acting like `str`,
and the `NumberBuf<B>` type owning the data inside the `B` type
(by default `String`).
By enabling the `smallnumberbuf` feature, the `SmallNumberBuf<LEN>` type is
defined as `NumberBuf<SmallVec<[u8; LEN]>>` (where `LEN=8` by default).

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
