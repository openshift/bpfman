# `DecodedChar`: Keeps track of the original byte length of a decoded character in the encoded source file.

[![CI](https://github.com/timothee-haudebourg/decoded-char/workflows/CI/badge.svg)](https://github.com/timothee-haudebourg/decoded-char/actions)
[![Crate informations](https://img.shields.io/crates/v/decoded-char.svg?style=flat-square)](https://crates.io/crates/decoded-char)
[![License](https://img.shields.io/crates/l/decoded-char.svg?style=flat-square)](https://github.com/timothee-haudebourg/decoded-char#license)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/decoded-char)

This is a very simple utility crate that provides a wrapper over `char`
values, `DecodedChar`, additionally storing the original byte length of the
character in the encoded source file.

It also provides wrappers around `char` iterators to produce `DecodedChar`
iterators from UTF-8/16 encoded sources.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
