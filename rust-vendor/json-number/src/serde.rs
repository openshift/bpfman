use crate::{Buffer, InvalidNumber, Number, NumberBuf};
use de::{Deserialize, Deserializer};
use ser::{Serialize, Serializer};
use serde::{de, forward_to_deserialize_any, ser};
use std::{fmt, marker::PhantomData};

/// Structure name used to serialize number with arbitrary precision.
///
/// This is the same used by `serde_json`, to ensure compatibility with this
/// crate.
const TOKEN: &str = "$serde_json::private::Number";

impl Serialize for Number {
	#[inline]
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		if self.has_decimal_point() {
			use serde::ser::SerializeStruct;
			let mut s = serializer.serialize_struct(TOKEN, 1)?;
			s.serialize_field(TOKEN, self.as_str())?;
			s.end()
		} else if let Some(v) = self.as_i64() {
			serializer.serialize_i64(v)
		} else if let Some(v) = self.as_u64() {
			serializer.serialize_u64(v)
		} else {
			Err(<S::Error as ser::Error>::custom("number too large"))
		}
	}
}

impl<B: Buffer> Serialize for NumberBuf<B> {
	#[inline]
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		self.as_number().serialize(serializer)
	}
}

impl<'de, B: Buffer> Deserialize<'de> for NumberBuf<B> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_any(Visitor(PhantomData))
	}
}

/// Number visitor.
pub struct Visitor<B>(PhantomData<B>);

impl<'de, B: Buffer> de::Visitor<'de> for Visitor<B> {
	type Value = NumberBuf<B>;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("JSON number")
	}

	#[inline]
	fn visit_u64<E: de::Error>(self, value: u64) -> Result<NumberBuf<B>, E> {
		Ok(value.into())
	}

	#[inline]
	fn visit_i64<E: de::Error>(self, value: i64) -> Result<NumberBuf<B>, E> {
		Ok(value.into())
	}

	#[inline]
	fn visit_f64<E: de::Error>(self, value: f64) -> Result<NumberBuf<B>, E> {
		NumberBuf::try_from(value)
			.map_err(|_| E::invalid_value(de::Unexpected::Float(value), &self))
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: de::MapAccess<'de>,
	{
		struct Key;

		impl<'de> Deserialize<'de> for Key {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: Deserializer<'de>,
			{
				struct KeyVisitor;

				impl<'de> de::Visitor<'de> for KeyVisitor {
					type Value = Key;

					fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
						formatter.write_str("a valid number field")
					}

					fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
					where
						E: de::Error,
					{
						if v == TOKEN {
							Ok(Key)
						} else {
							Err(serde::de::Error::custom("expected field with custom name"))
						}
					}
				}

				deserializer.deserialize_identifier(KeyVisitor)
			}
		}

		struct Value<B>(NumberBuf<B>);

		impl<'de, B: Buffer> Deserialize<'de> for Value<B> {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: Deserializer<'de>,
			{
				struct ValueVisitor<B>(PhantomData<B>);

				impl<'de, B: Buffer> de::Visitor<'de> for ValueVisitor<B> {
					type Value = Value<B>;

					fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
						formatter.write_str("string containing a JSON number")
					}

					fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
					where
						E: de::Error,
					{
						self.visit_string(v.to_owned())
					}

					fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
					where
						E: de::Error,
					{
						match NumberBuf::new(B::from_vec(v.into_bytes())) {
							Ok(v) => Ok(Value(v)),
							Err(InvalidNumber(bytes)) => Err(de::Error::custom(InvalidNumber(
								String::from_utf8(bytes.as_ref().to_owned()).unwrap(),
							))),
						}
					}
				}

				deserializer.deserialize_identifier(ValueVisitor(PhantomData))
			}
		}

		match map.next_key()? {
			Some(Key) => {
				let value: Value<B> = map.next_value()?;
				Ok(value.0)
			}
			None => Err(de::Error::invalid_type(de::Unexpected::Map, &self)),
		}
	}
}

/// Unexpected value that is not a number.
#[derive(Debug)]
pub struct Unexpected(String);

impl fmt::Display for Unexpected {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl std::error::Error for Unexpected {}

impl de::Error for Unexpected {
	fn custom<T>(msg: T) -> Self
	where
		T: fmt::Display,
	{
		Self(msg.to_string())
	}

	fn invalid_type(unexp: de::Unexpected, exp: &dyn de::Expected) -> Self {
		if let de::Unexpected::Unit = unexp {
			Self::custom(format_args!("invalid type: null, expected {}", exp))
		} else {
			Self::custom(format_args!("invalid type: {}, expected {}", unexp, exp))
		}
	}
}

impl<'de, B: Buffer> Deserializer<'de> for NumberBuf<B> {
	type Error = Unexpected;

	#[inline(always)]
	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.as_number().deserialize_any(visitor)
	}

	forward_to_deserialize_any! {
		bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
		bytes byte_buf option unit unit_struct seq tuple
		tuple_struct map struct newtype_struct enum identifier ignored_any
	}
}

impl<'de, 'n, B: Buffer> Deserializer<'de> for &'n NumberBuf<B> {
	type Error = Unexpected;

	#[inline(always)]
	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.as_number().deserialize_any(visitor)
	}

	forward_to_deserialize_any! {
		bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
		bytes byte_buf option unit unit_struct seq tuple
		tuple_struct map struct newtype_struct enum identifier ignored_any
	}
}

impl<'de, 'n> Deserializer<'de> for &'n Number {
	type Error = Unexpected;

	#[inline(always)]
	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		if let Some(u) = self.as_u64() {
			visitor.visit_u64(u)
		} else if let Some(i) = self.as_i64() {
			visitor.visit_i64(i)
		} else {
			visitor.visit_f64(self.as_f64_lossy())
		}
	}

	forward_to_deserialize_any! {
		bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
		bytes byte_buf option unit unit_struct seq tuple
		tuple_struct map struct newtype_struct enum identifier ignored_any
	}
}
