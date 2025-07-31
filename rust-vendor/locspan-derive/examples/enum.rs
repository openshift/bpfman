use locspan::{BorrowStripped, Loc, Location};
use locspan_derive::{
	StrippedEq, StrippedHash, StrippedOrd, StrippedPartialEq, StrippedPartialOrd,
};

#[derive(
	PartialEq, StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedOrd, StrippedHash, Debug,
)]
#[locspan(ignore(S, P))]
pub enum Foo<S, P> {
	A(#[locspan(deref_stripped)] Loc<usize, S, P>),
	B(#[locspan(stripped)] char, #[locspan(stripped)] usize),
	C(
		#[locspan(deref_stripped)] Loc<usize, S, P>,
		#[locspan(stripped)] char,
		u8,
	),
}

fn main() {
	let a = Foo::A(Loc(0, Location::new(0, ())));
	let b = Foo::A(Loc(0, Location::new(1, ())));

	assert_eq!(a.stripped(), b.stripped())
}
