use locspan::{BorrowStripped, Loc, Location};
use locspan_derive::*;

#[derive(StrippedPartialEq, Debug)]
pub struct Unit;

#[derive(StrippedPartialEq, Debug)]
pub struct A<T: Clone> {
	a: Loc<T, (), ()>,
}

#[derive(StrippedPartialEq, StrippedPartialOrd, Debug)]
pub struct B<T> {
	a: Loc<T, (), ()>,
	#[locspan(stripped)]
	b: usize,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Thing<T>(T);

#[derive(StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedHash, Debug)]
#[locspan(stripped(T), fixed(T))]
pub struct C<T> {
	#[locspan(unwrap_deref_stripped)]
	a: Option<Loc<Thing<T>, u32, ()>>,
}

#[derive(StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedHash, Debug)]
#[locspan(stripped(T), fixed(T))]
pub struct D<T> {
	#[locspan(unwrap_deref2_stripped)]
	a: Option<Loc<Loc<Thing<T>, u32, ()>, u32, ()>>,
}

#[derive(StrippedPartialEq, Debug)]
#[locspan(ignore(S, P))]
pub struct Foo<T: Clone, S, P>(Loc<T, S, P>);

fn main() {
	let a = C {
		a: Some(Loc(Thing(0u32), Location::new(0u32, ()))),
	};
	let b = C {
		a: Some(Loc(Thing(0u32), Location::new(1u32, ()))),
	};

	assert_eq!(a.stripped(), b.stripped());

	let a = D {
		a: Some(Loc(
			Loc(Thing(0u32), Location::new(1u32, ())),
			Location::new(0u32, ()),
		)),
	};
	let b = D {
		a: Some(Loc(
			Loc(Thing(0u32), Location::new(0u32, ())),
			Location::new(1u32, ()),
		)),
	};

	assert_eq!(a.stripped(), b.stripped());

	let a = Foo(Loc(0u32, Location::new(0u32, ())));
	let b = Foo(Loc(0u32, Location::new(1u32, ())));

	assert_eq!(a.stripped(), b.stripped())
}
