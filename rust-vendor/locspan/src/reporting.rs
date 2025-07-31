use crate::Location;

impl<F: Clone> Location<F> {
	#[inline(always)]
	pub fn as_primary_label(&self) -> codespan_reporting::diagnostic::Label<F> {
		codespan_reporting::diagnostic::Label::primary(self.file().clone(), self.span())
	}

	#[inline(always)]
	pub fn as_secondary_label(&self) -> codespan_reporting::diagnostic::Label<F> {
		codespan_reporting::diagnostic::Label::secondary(self.file().clone(), self.span())
	}
}

impl<F> Location<F> {
	#[inline(always)]
	pub fn into_primary_label(self) -> codespan_reporting::diagnostic::Label<F> {
		let (file, span) = self.into_parts();
		codespan_reporting::diagnostic::Label::primary(file, span)
	}

	#[inline(always)]
	pub fn into_secondary_label(self) -> codespan_reporting::diagnostic::Label<F> {
		let (file, span) = self.into_parts();
		codespan_reporting::diagnostic::Label::secondary(file, span)
	}
}
