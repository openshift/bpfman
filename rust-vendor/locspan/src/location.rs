use crate::{MaybeSpanned, Meta, Span, Spanned};

/// Syntax element location.
///
/// Provides a file identifier (of type `F`) and a [`Span`] in this file.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Location<F, S = Span> {
	/// File id.
	file: F,

	/// Span.
	span: S,
}

impl<F, S> Location<F, S> {
	/// Creates a new location referring to the given `span` in the given `file`.
	#[inline(always)]
	pub fn new(file: F, span: S) -> Self {
		Self { file, span }
	}

	/// Consumes this location and returns a pair
	/// containing the file and span.
	#[inline(always)]
	pub fn into_parts(self) -> (F, S) {
		(self.file, self.span)
	}

	/// Consumes this location and returns the file.
	#[inline(always)]
	pub fn into_file(self) -> F {
		self.file
	}

	/// Consumes this location and returns the span.
	#[inline(always)]
	pub fn into_span(self) -> S {
		self.span
	}

	/// Returns a reference to the file's identifier.
	#[inline(always)]
	pub fn file(&self) -> &F {
		&self.file
	}

	/// Returns a mutable reference to the file's identifier.
	#[inline(always)]
	pub fn file_mut(&mut self) -> &mut F {
		&mut self.file
	}

	/// Sets the file and returns the previous one.
	#[inline(always)]
	pub fn set_file(&mut self, mut file: F) -> F {
		std::mem::swap(&mut self.file, &mut file);
		file
	}

	/// Returns the `Span` in the file.
	#[inline(always)]
	pub fn span(&self) -> S
	where
		S: Clone,
	{
		self.span.clone()
	}

	/// Returns a mutable reference to the span.
	#[inline(always)]
	pub fn span_mut(&mut self) -> &mut S {
		&mut self.span
	}

	/// Sets the span and returns the previous one.
	#[inline(always)]
	pub fn set_span(&mut self, mut span: S) -> S {
		std::mem::swap(&mut self.span, &mut span);
		span
	}

	/// Maps the file identifier.
	#[inline(always)]
	pub fn map_file<G>(self, f: impl FnOnce(F) -> G) -> Location<G, S> {
		Location {
			file: f(self.file),
			span: self.span,
		}
	}

	/// Copies the span and borrows the file to create a new `Location<&F>`.
	#[inline(always)]
	pub fn borrow(&self) -> Location<&F, S>
	where
		S: Clone,
	{
		Location::new(&self.file, self.span.clone())
	}

	/// Converts the location.
	#[inline(always)]
	pub fn cast<G, P>(self) -> Location<G, P>
	where
		F: Into<G>,
		S: Into<P>,
	{
		Location::new(self.file.into(), self.span.into())
	}
}

impl<F> Location<F> {
	/// Sets the end of the location span to `end`, and returns itself.
	#[inline(always)]
	pub fn until(mut self, end: usize) -> Self {
		self.span.set_end(end);
		self
	}

	/// Append `span` to the location span, and returns itself.
	#[inline(always)]
	pub fn with(mut self, span: Span) -> Self {
		self.span.append(span);
		self
	}
}

impl<'a, F: Clone, S: Clone> Location<&'a F, S> {
	/// Clones the borrowed file to return a new `Location<F>`.
	#[inline(always)]
	pub fn cloned(&self) -> Location<F, S> {
		Location::new(self.file.clone(), self.span.clone())
	}
}

/// Value with a location.
pub trait Located {
	type File;
	type Span;

	fn location(&self) -> &Location<Self::File, Self::Span>;
}

impl<F, S> Located for Location<F, S> {
	type File = F;
	type Span = S;

	fn location(&self) -> &Location<Self::File, Self::Span> {
		self
	}
}

impl<T: Located> Spanned for T
where
	T::Span: Clone,
{
	type Span = T::Span;

	fn span(&self) -> Self::Span {
		self.location().span()
	}
}

/// Value with an optional location.
pub trait MaybeLocated {
	type File;
	type Span;

	fn optional_location(&self) -> Option<&Location<Self::File, Self::Span>>;
}

impl<T: MaybeLocated> MaybeSpanned for T
where
	T::Span: Clone,
{
	type Span = T::Span;

	fn optional_span(&self) -> Option<Self::Span> {
		self.optional_location().map(Location::span)
	}
}

impl<T: Located> MaybeLocated for T {
	type File = T::File;
	type Span = T::Span;

	fn optional_location(&self) -> Option<&Location<Self::File, Self::Span>> {
		Some(self.location())
	}
}

impl<T, F, S> Located for Meta<T, Location<F, S>> {
	type File = F;
	type Span = S;

	fn location(&self) -> &Location<Self::File, Self::Span> {
		self.metadata()
	}
}
