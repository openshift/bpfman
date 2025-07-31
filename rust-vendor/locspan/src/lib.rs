//! This library provides essentials types and trait to locate syntax elements.
//!
//! - `Span` describes a byte range in a source file, with an intuitive API to write lexers and parsers.
//! - `Location<F>` combines a `Span` with a file identifier `F` to pin point a syntactic element in a source file.
//! - `Meta<T, M>` wraps any value `T` with some metadata of type `M`.
//! - `Loc<T, F, S> = Meta<T, Location<F, S>>` wraps any value `T` and with its location.
//!
//! Extra traits are also provided to extend common types (`Option`, `Result`, etc.) with localization functions.
//! The crate integrates well with diagnostic reporting libraries such as
//! [`codespan-reporting`](https://crates.io/crates/codespan-reporting) to render beautiful error reports.
mod loc;
mod location;
mod meta;
mod span;
mod strip;

#[cfg(feature = "reporting")]
mod reporting;

#[cfg(feature = "serde")]
mod serde;

pub use loc::*;
pub use location::*;
pub use meta::*;
pub use span::*;
pub use strip::*;
