//! This is a very simple utility crate that provides a wrapper over `char`
//! values, `DecodedChar`, additionally storing the original byte length of the
//! character in the encoded source file.
//!
//! It also provides wrappers around `char` iterators to produce `DecodedChar`
//! iterators from UTF-8/16 encoded sources.
use std::borrow::Borrow;
use std::ops::Deref;

/// Decoded character.
///
/// A character and its original byte length in the encoded source file.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DecodedChar {
	/// Character.
	c: char,

	/// Byte length in the encoded source file.
	len: usize,
}

impl DecodedChar {
	/// Creates a new decoded character from its value, `c`,
	/// and its original byte length `len` in the encoded source file.
	#[inline(always)]
	pub fn new(c: char, len: usize) -> Self {
		Self { c, len }
	}

	/// Creates a new decoded character,
	/// decoded from an UTF-8 encoded source file.
	#[inline(always)]
	pub fn from_utf8(c: char) -> Self {
		Self {
			c,
			len: c.len_utf8(),
		}
	}

	/// Creates a new decoded character,
	/// decoded from an UTF-16 encoded source file.
	#[inline(always)]
	pub fn from_utf16(c: char) -> Self {
		Self {
			c,
			len: c.len_utf16(),
		}
	}

	/// Returns the character.
	#[inline(always)]
	pub fn chr(&self) -> char {
		self.c
	}

	/// Returns the original byte length of the character in the encoded source
	/// file.
	#[inline(always)]
	#[allow(clippy::len_without_is_empty)]
	pub fn len(&self) -> usize {
		self.len
	}

	/// Turns this `DecodedChar` into the underlying `char`.
	#[inline(always)]
	pub fn into_char(self) -> char {
		self.c
	}

	/// Turns this `DecodedChar` into the original byte length in the encoded
	/// source file.
	#[inline(always)]
	pub fn into_len(self) -> usize {
		self.len
	}
}

impl From<DecodedChar> for char {
	#[inline(always)]
	fn from(dc: DecodedChar) -> Self {
		dc.into_char()
	}
}

impl From<DecodedChar> for u32 {
	#[inline(always)]
	fn from(dc: DecodedChar) -> Self {
		dc.into_char().into()
	}
}

impl AsRef<char> for DecodedChar {
	#[inline(always)]
	fn as_ref(&self) -> &char {
		&self.c
	}
}

impl Borrow<char> for DecodedChar {
	#[inline(always)]
	fn borrow(&self) -> &char {
		&self.c
	}
}

impl Deref for DecodedChar {
	type Target = char;

	#[inline(always)]
	fn deref(&self) -> &char {
		&self.c
	}
}

/// Iterator wrapper around UTF-8 encoded sources.
#[derive(Clone, Debug)]
pub struct Utf8Decoded<C>(pub C);

impl<C> Utf8Decoded<C> {
	#[inline(always)]
	pub fn new(chars: C) -> Self {
		Self(chars)
	}
}

impl<C: Iterator<Item = char>> Iterator for Utf8Decoded<C> {
	type Item = DecodedChar;

	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(DecodedChar::from_utf8)
	}
}

/// Iterator wrapper around fallible UTF-8 encoded sources.
#[derive(Clone, Debug)]
pub struct FallibleUtf8Decoded<C>(pub C);

impl<C> FallibleUtf8Decoded<C> {
	#[inline(always)]
	pub fn new(chars: C) -> Self {
		Self(chars)
	}
}

impl<E, C: Iterator<Item = Result<char, E>>> Iterator for FallibleUtf8Decoded<C> {
	type Item = Result<DecodedChar, E>;

	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item> {
		self.0
			.next()
			.map(|result| result.map(DecodedChar::from_utf8))
	}
}

/// Iterator wrapper around UTF-16 encoded sources.
#[derive(Clone, Debug)]
pub struct Utf16Decoded<C>(pub C);

impl<C> Utf16Decoded<C> {
	#[inline(always)]
	pub fn new(chars: C) -> Self {
		Self(chars)
	}
}

impl<C: Iterator<Item = char>> Iterator for Utf16Decoded<C> {
	type Item = DecodedChar;

	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(DecodedChar::from_utf16)
	}
}

/// Iterator wrapper around fallible UTF-16 encoded sources.
#[derive(Clone, Debug)]
pub struct FallibleUtf16Decoded<C>(pub C);

impl<C> FallibleUtf16Decoded<C> {
	#[inline(always)]
	pub fn new(chars: C) -> Self {
		Self(chars)
	}
}

impl<E, C: Iterator<Item = Result<char, E>>> Iterator for FallibleUtf16Decoded<C> {
	type Item = Result<DecodedChar, E>;

	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item> {
		self.0
			.next()
			.map(|result| result.map(DecodedChar::from_utf16))
	}
}

/// Extension for `str` providing the `decoded_char` method.
pub trait DecodedChars {
	/// Returns an iterator over the UTF-8 decoded characters of the string,
	/// wrapped inside a `DecodedChar`.
	fn decoded_chars(&self) -> Utf8Decoded<std::str::Chars>;
}

impl DecodedChars for str {
	fn decoded_chars(&self) -> Utf8Decoded<std::str::Chars> {
		Utf8Decoded(self.chars())
	}
}
