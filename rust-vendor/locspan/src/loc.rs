use crate::{Location, Meta, Span};

/// Data with `Location` metadata.
///
/// This type alias is provided with a constructor function of the same name
/// so it is possible to build `Loc` values with `Loc(value, location)`.
pub type Loc<T, F, S = Span> = Meta<T, Location<F, S>>;

/// Build a data with `Location` metadata.
#[allow(non_snake_case)]
#[doc(hidden)]
#[inline(always)]
pub fn Loc<T, F, S>(t: T, location: Location<F, S>) -> Loc<T, F, S> {
	Meta(t, location)
}

impl<T, F, S> Loc<T, F, S> {
	/// Discards the value and returns its location.
	pub fn into_location(self) -> Location<F, S> {
		self.into_metadata()
	}

	/// Discards the value and returns its file.
	#[inline(always)]
	pub fn into_file(self) -> F {
		self.1.into_file()
	}

	/// Discards the value and returns its span.
	#[inline(always)]
	pub fn into_span(self) -> S {
		self.1.into_span()
	}

	/// Returns a reference to the value's location.
	#[inline(always)]
	pub fn location(&self) -> &Location<F, S> {
		self.metadata()
	}

	/// Returns a mutable reference to the value's location.
	#[inline(always)]
	pub fn location_mut(&mut self) -> &mut Location<F, S> {
		self.metadata_mut()
	}

	/// Returns the value's span.
	#[inline(always)]
	pub fn span(&self) -> S
	where
		S: Clone,
	{
		self.1.span()
	}

	/// Returns a mutable reference the value's span.
	#[inline(always)]
	pub fn span_mut(&mut self) -> &mut S {
		self.1.span_mut()
	}

	/// Sets the value's span and returns the previous one.
	#[inline(always)]
	pub fn set_span(&mut self, span: S) -> S {
		self.1.set_span(span)
	}

	/// Returns a reference to the value's source file.
	#[inline(always)]
	pub fn file(&self) -> &F {
		self.1.file()
	}

	/// Returns a mutable reference to the value's source file.
	#[inline(always)]
	pub fn file_mut(&mut self) -> &mut F {
		self.1.file_mut()
	}

	/// Sets the value's file and returns the previous one.
	#[inline(always)]
	pub fn set_file(&mut self, file: F) -> F {
		self.1.set_file(file)
	}

	/// Maps the value's location.
	#[inline(always)]
	pub fn map_location<G, U>(
		self,
		f: impl FnOnce(Location<F, S>) -> Location<G, U>,
	) -> Loc<T, G, U> {
		Loc(self.0, f(self.1))
	}

	/// Maps the value's location's file.
	#[inline(always)]
	pub fn map_file<G>(self, f: impl FnOnce(F) -> G) -> Loc<T, G, S> {
		Loc(self.0, self.1.map_file(f))
	}

	/// Borrows the file and clones the value.
	#[inline(always)]
	pub fn borrow_value_and_file(&self) -> Loc<&T, &F, S>
	where
		T: Clone,
		S: Clone,
	{
		Loc(&self.0, self.1.borrow())
	}

	/// Borrows the file and clones the value.
	#[inline(always)]
	pub fn borrow_file(&self) -> Loc<T, &F, S>
	where
		T: Clone,
		S: Clone,
	{
		Meta(self.0.clone(), self.1.borrow())
	}
}

impl<'f, T, F: Clone, S: Clone> Loc<T, &'f F, S> {
	/// Clones the value and the borrowed file to return a new `Loc<T, F>`.
	#[inline(always)]
	pub fn cloned_file(&self) -> Loc<T, F, S>
	where
		T: Clone,
	{
		Loc(self.0.clone(), self.1.cloned())
	}

	/// Clones the borrowed file and consumes the value to return a new `Loc<T, F>`.
	#[inline(always)]
	pub fn into_cloned_file(self) -> Loc<T, F, S> {
		Loc(self.0, self.1.cloned())
	}
}

impl<'t, 'f, T: Clone, F: Clone, S: Clone> Loc<&'t T, &'f F, S> {
	/// Clones the borrowed value and file to return a new `Loc<T, F>`.
	pub fn cloned(&self) -> Loc<T, F, S> {
		Loc(self.0.clone(), self.1.cloned())
	}
}
