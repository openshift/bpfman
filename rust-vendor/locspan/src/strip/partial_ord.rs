use super::{Stripped, StrippedPartialEq};
use crate::Meta;
use std::cmp::{Ordering, PartialOrd};

/// Defines the partial ordering of located values
/// without considering locations.
pub trait StrippedPartialOrd<U: ?Sized = Self>: StrippedPartialEq<U> {
	fn stripped_partial_cmp(&self, other: &U) -> Option<Ordering>;
}

impl<T: StrippedPartialOrd<U>, U> PartialOrd<Stripped<U>> for Stripped<T> {
	fn partial_cmp(&self, other: &Stripped<U>) -> Option<Ordering> {
		self.0.stripped_partial_cmp(&other.0)
	}
}

impl<T: StrippedPartialOrd<U>, U> StrippedPartialOrd<Stripped<U>> for Stripped<T> {
	fn stripped_partial_cmp(&self, other: &Stripped<U>) -> Option<Ordering> {
		self.0.stripped_partial_cmp(&other.0)
	}
}

impl<'u, 't, U, T: StrippedPartialOrd<U>> StrippedPartialOrd<&'u U> for &'t T {
	fn stripped_partial_cmp(&self, other: &&'u U) -> Option<Ordering> {
		T::stripped_partial_cmp(*self, *other)
	}
}

impl<U, N, T: StrippedPartialOrd<U>, M> StrippedPartialOrd<Meta<U, N>> for Meta<T, M> {
	fn stripped_partial_cmp(&self, other: &Meta<U, N>) -> Option<Ordering> {
		self.value().stripped_partial_cmp(other.value())
	}
}

impl<T: StrippedPartialOrd<U>, U> StrippedPartialOrd<Box<U>> for Box<T> {
	fn stripped_partial_cmp(&self, other: &Box<U>) -> Option<Ordering> {
		(**self).stripped_partial_cmp(&**other)
	}
}

impl<T: StrippedPartialOrd<U>, U> StrippedPartialOrd<Option<U>> for Option<T> {
	fn stripped_partial_cmp(&self, other: &Option<U>) -> Option<Ordering> {
		match (self, other) {
			(None, None) => Some(Ordering::Equal),
			(None, Some(_)) => Some(Ordering::Less),
			(Some(_), None) => Some(Ordering::Greater),
			(Some(a), Some(b)) => a.stripped_partial_cmp(b),
		}
	}
}

impl<T: StrippedPartialOrd<U>, U> StrippedPartialOrd<Vec<U>> for Vec<T> {
	fn stripped_partial_cmp(&self, other: &Vec<U>) -> Option<Ordering> {
		let mut self_iter = self.iter();
		let mut other_iter = other.iter();

		loop {
			match (self_iter.next(), other_iter.next()) {
				(Some(a), Some(b)) => match a.stripped_partial_cmp(b) {
					Some(Ordering::Equal) => (),
					cmp => break cmp,
				},
				(None, Some(_)) => break Some(Ordering::Less),
				(Some(_), None) => break Some(Ordering::Greater),
				(None, None) => break Some(Ordering::Equal),
			}
		}
	}
}
