use crate::Meta;
use std::{
	borrow,
	collections::{BTreeSet, HashSet},
	fmt,
	hash::Hash,
	ops,
};

mod eq;
mod hash;
mod ord;
mod partial_eq;
mod partial_ord;

pub use eq::*;
pub use hash::*;
pub use ord::*;
pub use partial_eq::*;
pub use partial_ord::*;

/// Type that can be stripped of its metadata.
pub trait Strip {
	type Stripped;

	fn strip(self) -> Self::Stripped;
}

impl<T: Strip, M> Strip for Meta<T, M> {
	type Stripped = T::Stripped;

	fn strip(self) -> Self::Stripped {
		self.0.strip()
	}
}

impl<T: Strip> Strip for Box<T> {
	type Stripped = Box<T::Stripped>;

	fn strip(self) -> Self::Stripped {
		Box::new((*self).strip())
	}
}

impl<T: Strip> Strip for Option<T> {
	type Stripped = Option<T::Stripped>;

	fn strip(self) -> Self::Stripped {
		self.map(T::strip)
	}
}

impl<T: Strip> Strip for Vec<T> {
	type Stripped = Vec<T::Stripped>;

	fn strip(self) -> Self::Stripped {
		self.into_iter().map(T::strip).collect()
	}
}

impl<T: Strip> Strip for HashSet<T>
where
	T::Stripped: Hash + Eq,
{
	type Stripped = HashSet<T::Stripped>;

	fn strip(self) -> Self::Stripped {
		self.into_iter().map(T::strip).collect()
	}
}

#[cfg(feature = "hashbrown")]
impl<T: Strip> Strip for hashbrown::HashSet<T>
where
	T::Stripped: Hash + Eq,
{
	type Stripped = hashbrown::HashSet<T::Stripped>;

	fn strip(self) -> Self::Stripped {
		self.into_iter().map(T::strip).collect()
	}
}

#[cfg(feature = "indexmap")]
impl<T: Strip> Strip for indexmap::IndexSet<T>
where
	T::Stripped: Hash + Eq,
{
	type Stripped = indexmap::IndexSet<T::Stripped>;

	fn strip(self) -> Self::Stripped {
		self.into_iter().map(T::strip).collect()
	}
}

impl<T: Strip> Strip for BTreeSet<T>
where
	T::Stripped: Ord,
{
	type Stripped = BTreeSet<T::Stripped>;

	fn strip(self) -> Self::Stripped {
		self.into_iter().map(T::strip).collect()
	}
}

pub trait BorrowStripped {
	fn stripped(&self) -> &Stripped<Self>;
}

impl<T> BorrowStripped for T {
	fn stripped(&self) -> &Stripped<Self> {
		unsafe { core::mem::transmute(self) }
	}
}

/// Wrapper to consider values without metadata.
///
/// This wrapper can be used in combination with the `Stripped*` traits such
/// as `StrippedPartialEq` to access and compare values ignoring code mapping
/// metadata.
///
/// An owned value can directly be wrapped.
/// Any reference `&T` can be safely converted into `&Stripped<T>` using the
/// [`BorrowStripped::stripped`] method.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Stripped<T: ?Sized>(pub T);

impl<T> Stripped<T> {
	#[inline]
	pub fn unwrap(self) -> T {
		self.0
	}
}

impl<T: fmt::Display> fmt::Display for Stripped<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl<T> ops::Deref for Stripped<T> {
	type Target = T;

	fn deref(&self) -> &T {
		&self.0
	}
}

impl<T> ops::DerefMut for Stripped<T> {
	fn deref_mut(&mut self) -> &mut T {
		&mut self.0
	}
}

impl<T> borrow::Borrow<T> for Stripped<T> {
	fn borrow(&self) -> &T {
		&self.0
	}
}

impl<T> borrow::BorrowMut<T> for Stripped<T> {
	fn borrow_mut(&mut self) -> &mut T {
		&mut self.0
	}
}

impl<T> AsRef<T> for Stripped<T> {
	fn as_ref(&self) -> &T {
		&self.0
	}
}

impl<T> AsMut<T> for Stripped<T> {
	fn as_mut(&mut self) -> &mut T {
		&mut self.0
	}
}

macro_rules! primitive {
	($($id:ident),*) => {
		$(
			impl Strip for $id {
				type Stripped = Self;

				fn strip(self) -> Self::Stripped {
					self
				}
			}

			impl<U> StrippedPartialEq<U> for $id where $id: PartialEq<U> {
				fn stripped_eq(&self, other: &U) -> bool {
					self == other
				}
			}

			impl StrippedEq for $id {}

			impl<U> StrippedPartialOrd<U> for $id where $id: PartialOrd<U> {
				fn stripped_partial_cmp(&self, other: &U) -> Option<std::cmp::Ordering> {
					self.partial_cmp(other)
				}
			}

			impl StrippedOrd for $id {
				fn stripped_cmp(&self, other: &Self) -> std::cmp::Ordering {
					self.cmp(other)
				}
			}

			impl StrippedHash for $id {
				fn stripped_hash<H: std::hash::Hasher>(&self, state: &mut H) {
					self.hash(state)
				}
			}
		)*
	};
}

primitive! {
	bool,
	u8,
	u16,
	u32,
	u64,
	i8,
	i16,
	i32,
	i64,
	usize,
	isize,
	char,
	String
}

macro_rules! float {
	($($id:ident),*) => {
		$(
			impl Strip for $id {
				type Stripped = Self;

				fn strip(self) -> Self::Stripped {
					self
				}
			}

			impl<U> StrippedPartialEq<U> for $id where $id: PartialEq<U> {
				fn stripped_eq(&self, other: &U) -> bool {
					self == other
				}
			}

			impl<U> StrippedPartialOrd<U> for $id where $id: PartialOrd<U> {
				fn stripped_partial_cmp(&self, other: &U) -> Option<std::cmp::Ordering> {
					self.partial_cmp(other)
				}
			}
		)*
	};
}

float! {
	f32,
	f64
}
