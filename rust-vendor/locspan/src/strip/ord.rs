use super::{Stripped, StrippedEq, StrippedPartialOrd};
use crate::Meta;
use std::cmp::{Ord, Ordering};

/// Defines the partial ordering of located values
/// without considering locations.
pub trait StrippedOrd: StrippedEq + StrippedPartialOrd {
	fn stripped_cmp(&self, other: &Self) -> Ordering;
}

impl<T: StrippedOrd> Ord for Stripped<T> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.0.stripped_cmp(&other.0)
	}
}

impl<T: StrippedOrd> StrippedOrd for Stripped<T> {
	fn stripped_cmp(&self, other: &Self) -> Ordering {
		self.0.stripped_cmp(&other.0)
	}
}

impl<'t, T: StrippedOrd> StrippedOrd for &'t T {
	fn stripped_cmp(&self, other: &Self) -> Ordering {
		T::stripped_cmp(*self, *other)
	}
}

impl<T: StrippedOrd, M> StrippedOrd for Meta<T, M> {
	fn stripped_cmp(&self, other: &Self) -> Ordering {
		self.value().stripped_cmp(other.value())
	}
}

impl<T: StrippedOrd> StrippedOrd for Box<T> {
	fn stripped_cmp(&self, other: &Self) -> Ordering {
		(**self).stripped_cmp(&**other)
	}
}

impl<T: StrippedOrd> StrippedOrd for Option<T> {
	fn stripped_cmp(&self, other: &Self) -> Ordering {
		match (self, other) {
			(None, None) => Ordering::Equal,
			(None, Some(_)) => Ordering::Less,
			(Some(_), None) => Ordering::Greater,
			(Some(a), Some(b)) => a.stripped_cmp(b),
		}
	}
}

impl<T: StrippedOrd> StrippedOrd for Vec<T> {
	fn stripped_cmp(&self, other: &Self) -> Ordering {
		let mut self_iter = self.iter();
		let mut other_iter = other.iter();

		loop {
			match (self_iter.next(), other_iter.next()) {
				(Some(a), Some(b)) => match a.stripped_cmp(b) {
					Ordering::Equal => (),
					cmp => break cmp,
				},
				(None, Some(_)) => break Ordering::Less,
				(Some(_), None) => break Ordering::Greater,
				(None, None) => break Ordering::Equal,
			}
		}
	}
}
