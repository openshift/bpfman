//! This is a simple library for parsing and storing JSON numbers according
//! to the [JSON specification](https://www.json.org/json-en.html).
//! It provides two types, the unsized `Number` type acting like `str`,
//! and the `NumberBuf<B>` type owning the data inside the `B` type
//! (by default `Vec<u8>`).
//!
//! # Features
//!
//! ## Store small owned numbers on the stack
//!
//! By enabling the `smallnumberbuf` feature, the `SmallNumberBuf<LEN>` type is
//! defined as `NumberBuf<SmallVec<[u8; LEN]>>` (where `LEN=8` by default)
//! thanks to the [`smallvec`](https://crates.io/crates/smallvec) crate.
//!
//! ## Serde support
//!
//! Enable the `serde` feature to add `Serialize`, `Deserialize` and
//! `Deserializer` implementations to `NumberBuf`.
use std::borrow::{Borrow, ToOwned};
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

/// `serde` support.
#[cfg(feature = "serde")]
pub mod serde;

/// `serde_json` support.
#[cfg(feature = "serde_json")]
pub mod serde_json;

#[cfg(feature = "smallnumberbuf")]
mod smallnumberbuf {
	use super::*;
	use smallvec::SmallVec;

	/// JSON number buffer based on [`SmallVec`](smallvec::SmallVec).
	pub type SmallNumberBuf<const LEN: usize = 8> = NumberBuf<SmallVec<[u8; LEN]>>;

	unsafe impl<A: smallvec::Array<Item = u8>> crate::Buffer for SmallVec<A> {
		fn from_vec(bytes: Vec<u8>) -> Self {
			bytes.into()
		}

		fn from_bytes(bytes: &[u8]) -> Self {
			bytes.into()
		}
	}
}

#[cfg(feature = "smallnumberbuf")]
pub use smallnumberbuf::*;

/// Invalid number error.
///
/// The inner value is the data failed to be parsed.
#[derive(Clone, Copy, Debug)]
pub struct InvalidNumber<T>(pub T);

impl<T: fmt::Display> fmt::Display for InvalidNumber<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "invalid JSON number: {}", self.0)
	}
}

impl<T: fmt::Display + fmt::Debug> std::error::Error for InvalidNumber<T> {}

/// Number sign.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Sign {
	Negative,
	Zero,
	Positive,
}

impl Sign {
	/// Checks if the number is zero.
	#[inline(always)]
	pub fn is_zero(&self) -> bool {
		matches!(self, Self::Zero)
	}

	/// Checks if the number is non positive (negative or zero).
	#[inline(always)]
	pub fn is_non_positive(&self) -> bool {
		matches!(self, Self::Negative | Self::Zero)
	}

	/// Checks if the number is non negative (positive or zero).
	#[inline(always)]
	pub fn is_non_negative(&self) -> bool {
		matches!(self, Self::Positive | Self::Zero)
	}

	/// Checks if the number is strictly positive (non zero nor negative).
	#[inline(always)]
	pub fn is_positive(&self) -> bool {
		matches!(self, Self::Positive)
	}

	/// Checks if the number is strictly negative (non zero nor positive).
	#[inline(always)]
	pub fn is_negative(&self) -> bool {
		matches!(self, Self::Negative)
	}
}

/// Lexical JSON number.
///
/// This hold the lexical representation of a JSON number.
/// All the comparison operations are done on this *lexical* representation,
/// meaning that `1` is actually greater than `0.1e+80` for instance.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Number {
	data: [u8],
}

impl Number {
	/// Creates a new number by parsing the given input `data`.
	pub fn new<B: AsRef<[u8]> + ?Sized>(data: &B) -> Result<&Number, InvalidNumber<&B>> {
		let s = data.as_ref();

		enum State {
			Init,
			FirstDigit,
			Zero,
			NonZero,
			FractionalFirst,
			FractionalRest,
			ExponentSign,
			ExponentFirst,
			ExponentRest,
		}

		let mut state = State::Init;

		for b in s {
			match state {
				State::Init => match *b {
					b'-' => state = State::FirstDigit,
					b'0' => state = State::Zero,
					b'1'..=b'9' => state = State::NonZero,
					_ => return Err(InvalidNumber(data)),
				},
				State::FirstDigit => match *b {
					b'0' => state = State::Zero,
					b'1'..=b'9' => state = State::NonZero,
					_ => return Err(InvalidNumber(data)),
				},
				State::Zero => match *b {
					b'.' => state = State::FractionalFirst,
					b'e' | b'E' => state = State::ExponentSign,
					_ => return Err(InvalidNumber(data)),
				},
				State::NonZero => match *b {
					b'0'..=b'9' => state = State::NonZero,
					b'.' => state = State::FractionalFirst,
					b'e' | b'E' => state = State::ExponentSign,
					_ => return Err(InvalidNumber(data)),
				},
				State::FractionalFirst => match *b {
					b'0'..=b'9' => state = State::FractionalRest,
					_ => return Err(InvalidNumber(data)),
				},
				State::FractionalRest => match *b {
					b'0'..=b'9' => state = State::FractionalRest,
					b'e' | b'E' => state = State::ExponentSign,
					_ => return Err(InvalidNumber(data)),
				},
				State::ExponentSign => match *b {
					b'+' | b'-' => state = State::ExponentFirst,
					b'0'..=b'9' => state = State::ExponentRest,
					_ => return Err(InvalidNumber(data)),
				},
				State::ExponentFirst => match *b {
					b'0'..=b'9' => state = State::ExponentRest,
					_ => return Err(InvalidNumber(data)),
				},
				State::ExponentRest => match *b {
					b'0'..=b'9' => state = State::ExponentRest,
					_ => return Err(InvalidNumber(data)),
				},
			}
		}

		if matches!(
			state,
			State::Zero | State::NonZero | State::FractionalRest | State::ExponentRest
		) {
			Ok(unsafe { Self::new_unchecked(s) })
		} else {
			Err(InvalidNumber(data))
		}
	}

	/// Creates a new number without parsing the given input `data`.
	///
	/// ## Safety
	///
	/// The `data` input **must** be a valid JSON number.
	#[inline(always)]
	pub unsafe fn new_unchecked<B: AsRef<[u8]> + ?Sized>(data: &B) -> &Number {
		std::mem::transmute(data.as_ref())
	}

	#[inline(always)]
	pub fn as_str(&self) -> &str {
		unsafe {
			// safe because `self.data` is always a valid UTF-8 sequence.
			std::str::from_utf8_unchecked(&self.data)
		}
	}

	pub fn trimmed(&self) -> &Self {
		let mut end = 1;
		let mut i = 1;
		let mut fractional_part = false;
		while i < self.data.len() {
			match self.data[i] {
				b'0' if fractional_part => (),
				b'.' => fractional_part = true,
				_ => end = i + 1,
			}

			i += 1
		}

		unsafe { Self::new_unchecked(&self.data[0..end]) }
	}

	/// Checks if the number is equal to zero (`0`).
	///
	/// This include every lexical representation where
	/// the decimal and fraction part are composed of only
	/// `0`, maybe preceded with `-`, and an arbitrary exponent part.
	#[inline(always)]
	pub fn is_zero(&self) -> bool {
		for b in &self.data {
			match b {
				b'-' | b'0' | b'.' => (),
				b'e' | b'E' => break,
				_ => return false,
			}
		}

		true
	}

	/// Returns the sign of the number.
	pub fn sign(&self) -> Sign {
		let mut non_negative = true;

		for b in &self.data {
			match b {
				b'-' => non_negative = false,
				b'0' | b'.' => (),
				b'e' | b'E' => break,
				_ => {
					return if non_negative {
						Sign::Positive
					} else {
						Sign::Negative
					}
				}
			}
		}

		Sign::Zero
	}

	/// Checks if the number is non positive (negative or zero).
	#[inline(always)]
	pub fn is_non_positive(&self) -> bool {
		self.sign().is_non_positive()
	}

	/// Checks if the number is non negative (positive or zero).
	#[inline(always)]
	pub fn is_non_negative(&self) -> bool {
		self.sign().is_non_negative()
	}

	/// Checks if the number is strictly positive (non zero nor negative).
	#[inline(always)]
	pub fn is_positive(&self) -> bool {
		self.sign().is_positive()
	}

	/// Checks if the number is strictly negative (non zero nor positive).
	#[inline(always)]
	pub fn is_negative(&self) -> bool {
		self.sign().is_negative()
	}

	/// Checks if the number has a decimal point.
	#[inline(always)]
	pub fn has_decimal_point(&self) -> bool {
		self.data.contains(&b'.')
	}

	/// Checks if the number has a fraction part.
	///
	/// This is an alias for [`has_decimal_point`](Self::has_decimal_point).
	#[inline(always)]
	pub fn has_fraction(&self) -> bool {
		self.has_decimal_point()
	}

	/// Checks if the number has an exponent part.
	#[inline(always)]
	pub fn has_exponent(&self) -> bool {
		for b in &self.data {
			if matches!(b, b'e' | b'E') {
				return true;
			}
		}

		false
	}

	#[inline(always)]
	pub fn is_i32(&self) -> bool {
		self.as_i32().is_some()
	}

	#[inline(always)]
	pub fn is_i64(&self) -> bool {
		self.as_i64().is_some()
	}

	#[inline(always)]
	pub fn is_u32(&self) -> bool {
		self.as_u32().is_some()
	}

	#[inline(always)]
	pub fn is_u64(&self) -> bool {
		self.as_u64().is_some()
	}

	#[inline(always)]
	pub fn as_i32(&self) -> Option<i32> {
		self.as_str().parse().ok()
	}

	#[inline(always)]
	pub fn as_i64(&self) -> Option<i64> {
		self.as_str().parse().ok()
	}

	#[inline(always)]
	pub fn as_u32(&self) -> Option<u32> {
		self.as_str().parse().ok()
	}

	#[inline(always)]
	pub fn as_u64(&self) -> Option<u64> {
		self.as_str().parse().ok()
	}

	#[inline(always)]
	pub fn as_f32_lossy(&self) -> f32 {
		lexical::parse_with_options::<_, _, { lexical::format::JSON }>(
			self.as_bytes(),
			&LOSSY_PARSE_FLOAT,
		)
		.unwrap()
	}

	/// Returns the number as a `f32` only if the operation does not induce
	/// imprecisions/approximations.
	///
	/// This operation is expensive as it requires allocating a new number
	/// buffer to check the decimal representation of the generated `f32`.
	#[inline(always)]
	pub fn as_f32_lossless(&self) -> Option<f32> {
		let f = self.as_f32_lossy();
		let n: NumberBuf = f.try_into().unwrap();
		eprintln!("n = {n} = {f}");
		if n.as_number() == self.trimmed() {
			Some(f)
		} else {
			None
		}
	}

	#[inline(always)]
	pub fn as_f64_lossy(&self) -> f64 {
		lexical::parse_with_options::<_, _, { lexical::format::JSON }>(
			self.as_bytes(),
			&LOSSY_PARSE_FLOAT,
		)
		.unwrap()
	}

	/// Returns the number as a `f64` only if the operation does not induce
	/// imprecisions/approximations.
	///
	/// This operation is expensive as it requires allocating a new number
	/// buffer to check the decimal representation of the generated `f64`.
	#[inline(always)]
	pub fn as_f64_lossless(&self) -> Option<f64> {
		let f = self.as_f64_lossy();
		let n: NumberBuf = f.try_into().unwrap();
		if n.as_number() == self {
			Some(f)
		} else {
			None
		}
	}

	/// Returns the canonical representation of this number according to
	/// [RFC8785](https://www.rfc-editor.org/rfc/rfc8785#name-serialization-of-numbers).
	#[cfg(feature = "canonical")]
	pub fn canonical_with<'b>(&self, buffer: &'b mut ryu_js::Buffer) -> &'b Number {
		unsafe { Number::new_unchecked(buffer.format_finite(self.as_f64_lossy())) }
	}

	/// Returns the canonical representation of this number according to
	/// [RFC8785](https://www.rfc-editor.org/rfc/rfc8785#name-serialization-of-numbers).
	#[cfg(feature = "canonical")]
	pub fn canonical(&self) -> NumberBuf {
		let mut buffer = ryu_js::Buffer::new();
		self.canonical_with(&mut buffer).to_owned()
	}
}

const LOSSY_PARSE_FLOAT: lexical::ParseFloatOptions =
	lexical::ParseFloatOptions::builder()
		.lossy(true)
		.build_unchecked()
;

impl Deref for Number {
	type Target = str;

	#[inline(always)]
	fn deref(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<str> for Number {
	#[inline(always)]
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl Borrow<str> for Number {
	#[inline(always)]
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<[u8]> for Number {
	#[inline(always)]
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl Borrow<[u8]> for Number {
	#[inline(always)]
	fn borrow(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl<'a> TryFrom<&'a str> for &'a Number {
	type Error = InvalidNumber<&'a str>;

	#[inline(always)]
	fn try_from(s: &'a str) -> Result<&'a Number, InvalidNumber<&'a str>> {
		Number::new(s)
	}
}

impl ToOwned for Number {
	type Owned = NumberBuf;

	fn to_owned(&self) -> Self::Owned {
		unsafe { NumberBuf::new_unchecked(self.as_bytes().to_owned()) }
	}
}

impl fmt::Display for Number {
	#[inline(always)]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl fmt::Debug for Number {
	#[inline(always)]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

/// Buffer type.
///
/// # Safety
///
/// The `AsRef<[u8]>` implementation *must* return the bytes provided using
/// the `from_bytes` and `from_vec` constructor functions.
pub unsafe trait Buffer: AsRef<[u8]> {
	fn from_bytes(bytes: &[u8]) -> Self;

	fn from_vec(bytes: Vec<u8>) -> Self;
}

unsafe impl Buffer for Vec<u8> {
	fn from_bytes(bytes: &[u8]) -> Self {
		bytes.into()
	}

	fn from_vec(bytes: Vec<u8>) -> Self {
		bytes
	}
}

/// JSON number buffer.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NumberBuf<B = Vec<u8>> {
	data: B,
}

impl<B> NumberBuf<B> {
	/// Creates a new number buffer by parsing the given input `data` buffer.
	#[inline(always)]
	pub fn new(data: B) -> Result<Self, InvalidNumber<B>>
	where
		B: AsRef<[u8]>,
	{
		match Number::new(&data) {
			Ok(_) => Ok(NumberBuf { data }),
			Err(_) => Err(InvalidNumber(data)),
		}
	}

	/// Creates a new number buffer from the given input `data` buffer.
	///
	/// ## Safety
	///
	/// The input `data` **must** hold a valid JSON number string.
	#[inline(always)]
	pub unsafe fn new_unchecked(data: B) -> Self {
		NumberBuf { data }
	}

	/// Creates a number buffer from the given `number`.
	#[inline(always)]
	pub fn from_number(n: &Number) -> Self
	where
		B: FromIterator<u8>,
	{
		unsafe { NumberBuf::new_unchecked(n.bytes().collect()) }
	}

	#[inline(always)]
	pub fn buffer(&self) -> &B {
		&self.data
	}

	#[inline(always)]
	pub fn into_buffer(self) -> B {
		self.data
	}
}

impl NumberBuf<String> {
	#[inline(always)]
	pub fn into_string(self) -> String {
		self.data
	}

	#[inline(always)]
	pub fn into_bytes(self) -> Vec<u8> {
		self.data.into_bytes()
	}
}

impl<B: Buffer> NumberBuf<B> {
	#[inline(always)]
	pub fn as_number(&self) -> &Number {
		unsafe { Number::new_unchecked(&self.data) }
	}
}

impl<B: Buffer> FromStr for NumberBuf<B> {
	type Err = InvalidNumber<B>;

	#[inline(always)]
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::new(B::from_bytes(s.as_bytes()))
	}
}

impl<B: Buffer> Deref for NumberBuf<B> {
	type Target = Number;

	#[inline(always)]
	fn deref(&self) -> &Number {
		self.as_number()
	}
}

impl<B: Buffer> AsRef<Number> for NumberBuf<B> {
	#[inline(always)]
	fn as_ref(&self) -> &Number {
		self.as_number()
	}
}

impl<B: Buffer> Borrow<Number> for NumberBuf<B> {
	#[inline(always)]
	fn borrow(&self) -> &Number {
		self.as_number()
	}
}

impl<B: Buffer> AsRef<str> for NumberBuf<B> {
	#[inline(always)]
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl<B: Buffer> Borrow<str> for NumberBuf<B> {
	#[inline(always)]
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl<B: Buffer> AsRef<[u8]> for NumberBuf<B> {
	#[inline(always)]
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl<B: Buffer> Borrow<[u8]> for NumberBuf<B> {
	#[inline(always)]
	fn borrow(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl<B: Buffer> fmt::Display for NumberBuf<B> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<B: Buffer> fmt::Debug for NumberBuf<B> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

macro_rules! impl_from_int {
	($($ty:ty),*) => {
		$(
			impl<B: Buffer> From<$ty> for NumberBuf<B> {
				#[inline(always)]
				fn from(i: $ty) -> Self {
					unsafe {
						Self::new_unchecked(B::from_vec(lexical::to_string(i).into_bytes()))
					}
				}
			}
		)*
	};
}

/// Float conversion error.
#[derive(Clone, Copy, Debug)]
pub enum TryFromFloatError {
	/// The float was Nan, which is not a JSON number.
	Nan,

	/// The float was not finite, and hence not a JSON number.
	Infinite,
}

const WRITE_FLOAT: lexical::WriteFloatOptions =
	lexical::WriteFloatOptions::builder()
		.trim_floats(true)
		.exponent(b'e')
		.build_unchecked()
;

macro_rules! impl_try_from_float {
	($($ty:ty),*) => {
		$(
			impl<B: Buffer> TryFrom<$ty> for NumberBuf<B> {
				type Error = TryFromFloatError;

				#[inline(always)]
				fn try_from(f: $ty) -> Result<Self, Self::Error> {
					if f.is_finite() {
						Ok(unsafe {
							Self::new_unchecked(B::from_vec(lexical::to_string_with_options::<_, {lexical::format::JSON}>(f, &WRITE_FLOAT).into_bytes()))
						})
					} else if f.is_nan() {
						Err(TryFromFloatError::Nan)
					} else {
						Err(TryFromFloatError::Infinite)
					}
				}
			}
		)*
	};
}

impl_from_int!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize);
impl_try_from_float!(f32, f64);

#[cfg(test)]
mod tests {
	use super::*;

	fn trimming_test(a: &str, b: &str) {
		let a = Number::new(a).unwrap();
		let b = Number::new(b).unwrap();
		assert_eq!(a.trimmed(), b)
	}

	#[test]
	fn trimming() {
		trimming_test("0", "0");
		trimming_test("0.0", "0");
		trimming_test("1.0", "1");
		trimming_test("1.0", "1");
		trimming_test("1.1", "1.1");
		trimming_test("1.10000", "1.1");
		trimming_test("100.0", "100");
		trimming_test("100.1000", "100.1");
	}

	macro_rules! positive_tests {
		{ $($id:ident: $input:literal),* } => {
			$(
				#[test]
				fn $id () {
					assert!(Number::new($input).is_ok())
				}
			)*
		};
	}

	macro_rules! negative_tests {
		{ $($id:ident: $input:literal),* } => {
			$(
				#[test]
				fn $id () {
					assert!(Number::new($input).is_err())
				}
			)*
		};
	}

	macro_rules! sign_tests {
		{ $($id:ident: $input:literal => $sign:ident),* } => {
			$(
				#[test]
				fn $id () {
					assert_eq!(Number::new($input).unwrap().sign(), Sign::$sign)
				}
			)*
		};
	}

	macro_rules! canonical_tests {
		{ $($id:ident: $input:literal => $output:literal),* } => {
			$(
				#[cfg(feature="canonical")]
				#[test]
				fn $id () {
					assert_eq!(Number::new($input).unwrap().canonical().as_number(), Number::new($output).unwrap())
				}
			)*
		};
	}

	positive_tests! {
		pos_01: "0",
		pos_02: "-0",
		pos_03: "123",
		pos_04: "1.23",
		pos_05: "-12.34",
		pos_06: "12.34e+56",
		pos_07: "12.34E-56",
		pos_08: "0.0000"
	}

	negative_tests! {
		neg_01: "",
		neg_02: "00",
		neg_03: "01",
		neg_04: "-00",
		neg_05: "-01",
		neg_06: "0.000e+-1",
		neg_07: "12.34E-56abc",
		neg_08: "1.",
		neg_09: "12.34e",
		neg_10: "12.34e+",
		neg_11: "12.34E-"
	}

	sign_tests! {
		sign_zero_01: "0" => Zero,
		sign_zero_02: "-0" => Zero,
		sign_zero_03: "0.0" => Zero,
		sign_zero_04: "0.0e12" => Zero,
		sign_zero_05: "-0.0E-12" => Zero,
		sign_zero_06: "-0.00000" => Zero
	}

	sign_tests! {
		sign_pos_01: "1" => Positive,
		sign_pos_02: "0.1" => Positive,
		sign_pos_03: "0.01e23" => Positive,
		sign_pos_04: "1.0E-23" => Positive,
		sign_pos_05: "0.00001" => Positive
	}

	sign_tests! {
		sign_neg_01: "-1" => Negative,
		sign_neg_02: "-0.1" => Negative,
		sign_neg_03: "-0.01e23" => Negative,
		sign_neg_04: "-1.0E-23" => Negative,
		sign_neg_05: "-0.00001" => Negative
	}

	canonical_tests! {
		canonical_01: "-0.0000" => "0",
		canonical_02: "0.00000000028" => "2.8e-10"
	}
}
