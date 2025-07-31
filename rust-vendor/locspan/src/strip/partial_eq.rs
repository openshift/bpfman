use super::Stripped;
use crate::Meta;
use std::collections::HashMap;
use std::hash::Hash;

/// Defines the equality of values
/// without considering the metadata.
///
/// ## Example
///
/// ```
/// use locspan::{Loc, Span, Location, StrippedPartialEq, BorrowStripped};
///
/// #[derive(PartialEq, Eq, Debug)]
/// struct MyValue(u32);
///
/// impl StrippedPartialEq for MyValue {
///   fn stripped_eq(&self, other: &Self) -> bool {
///     self == other
///   }
/// }
///
/// let a = Loc(MyValue(0), Location::new("a", Span::new(0, 1)));
/// let b = Loc(MyValue(0), Location::new("b", Span::new(2, 4)));
///
/// // `a` and `b` are not equals,
/// // because their associated `Location`s are different.
/// assert_ne!(a, b);
///
/// // However using `BorrowStripped::stripped` we can
/// // compare the inner values regardless of the locations.
/// assert_eq!(a.stripped(), b.stripped());
/// ```
pub trait StrippedPartialEq<U: ?Sized = Self> {
	fn stripped_eq(&self, other: &U) -> bool;
}

impl<T: StrippedPartialEq<U>, U> PartialEq<Stripped<U>> for Stripped<T> {
	fn eq(&self, other: &Stripped<U>) -> bool {
		self.0.stripped_eq(&other.0)
	}
}

impl<T: StrippedPartialEq<U>, U> StrippedPartialEq<Stripped<U>> for Stripped<T> {
	fn stripped_eq(&self, other: &Stripped<U>) -> bool {
		self.0.stripped_eq(&other.0)
	}
}

impl<'u, 't, U, T: StrippedPartialEq<U>> StrippedPartialEq<&'u U> for &'t T {
	fn stripped_eq(&self, other: &&'u U) -> bool {
		T::stripped_eq(*self, *other)
	}
}

impl<U, N, T: StrippedPartialEq<U>, M> StrippedPartialEq<Meta<U, N>> for Meta<T, M> {
	fn stripped_eq(&self, other: &Meta<U, N>) -> bool {
		self.value().stripped_eq(other.value())
	}
}

impl<T: StrippedPartialEq<U>, U> StrippedPartialEq<Box<U>> for Box<T> {
	fn stripped_eq(&self, other: &Box<U>) -> bool {
		(**self).stripped_eq(&**other)
	}
}

impl<T: StrippedPartialEq<U>, U> StrippedPartialEq<Option<U>> for Option<T> {
	fn stripped_eq(&self, other: &Option<U>) -> bool {
		match (self, other) {
			(Some(a), Some(b)) => a.stripped_eq(b),
			(None, None) => true,
			_ => false,
		}
	}
}

impl<T: StrippedPartialEq<U>, U> StrippedPartialEq<Vec<U>> for Vec<T> {
	fn stripped_eq(&self, other: &Vec<U>) -> bool {
		self.len() == other.len() && self.iter().zip(other).all(|(a, b)| a.stripped_eq(b))
	}
}

impl<K: Eq + Hash, V: StrippedPartialEq<W>, W> StrippedPartialEq<HashMap<K, W>> for HashMap<K, V> {
	fn stripped_eq(&self, other: &HashMap<K, W>) -> bool {
		self.len() == other.len()
			&& self
				.iter()
				.all(|(key, value)| other.get(key).map_or(false, |v| value.stripped_eq(v)))
	}
}

#[cfg(feature = "hashbrown")]
impl<K: Eq + Hash, V: StrippedPartialEq<W>, W> StrippedPartialEq<hashbrown::HashMap<K, W>>
	for hashbrown::HashMap<K, V>
{
	fn stripped_eq(&self, other: &hashbrown::HashMap<K, W>) -> bool {
		self.len() == other.len()
			&& self
				.iter()
				.all(|(key, value)| other.get(key).map_or(false, |v| value.stripped_eq(v)))
	}
}

#[cfg(feature = "indexmap")]
impl<K: Eq + Hash, V: StrippedPartialEq<W>, W> StrippedPartialEq<indexmap::IndexMap<K, W>>
	for indexmap::IndexMap<K, V>
{
	fn stripped_eq(&self, other: &indexmap::IndexMap<K, W>) -> bool {
		self.len() == other.len()
			&& self
				.iter()
				.all(|(key, value)| other.get(key).map_or(false, |v| value.stripped_eq(v)))
	}
}
