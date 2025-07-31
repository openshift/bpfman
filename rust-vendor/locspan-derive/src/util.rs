use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use std::iter;
use std::slice;
use syn::{
	punctuated::Punctuated, token::Comma, AngleBracketedGenericArguments, AttrStyle, Attribute,
	ExprPath, GenericArgument, GenericParam, Generics, Path, PathArguments, PathSegment, Type,
	TypeParam, TypePath,
};

use crate::ParamConfig;

pub trait TryCollect: Iterator<Item = Result<Self::OkItem, Self::Error>> {
	type OkItem;
	type Error;

	fn try_collect(self) -> Result<Vec<Self::OkItem>, Self::Error>;
}

impl<T, E, I: Iterator<Item = Result<T, E>>> TryCollect for I {
	type Error = E;
	type OkItem = T;

	fn try_collect(self) -> Result<Vec<Self::OkItem>, Self::Error> {
		let mut result = Vec::with_capacity(self.size_hint().0);

		for item in self {
			result.push(item?)
		}

		Ok(result)
	}
}

pub fn simple_path<'a>(
	prefix: impl 'a + IntoIterator<Item = &'a str>,
	name: &str,
	arguments: syn::PathArguments,
) -> syn::Path {
	syn::Path {
		leading_colon: Some(syn::token::Colon2::default()),
		segments: prefix
			.into_iter()
			.map(|name| syn::PathSegment {
				ident: syn::Ident::new(name, proc_macro2::Span::call_site()),
				arguments: syn::PathArguments::None,
			})
			.chain(Some(syn::PathSegment {
				ident: syn::Ident::new(name, proc_macro2::Span::call_site()),
				arguments,
			}))
			.collect(),
	}
}

fn type_ref(ident: Ident) -> Type {
	let mut segments = Punctuated::new();
	segments.push_value(PathSegment {
		ident,
		arguments: PathArguments::None,
	});

	Type::Path(TypePath {
		qself: None,
		path: Path {
			leading_colon: None,
			segments,
		},
	})
}

fn const_ref(ident: Ident) -> syn::Expr {
	let mut segments = Punctuated::new();
	segments.push_value(PathSegment {
		ident,
		arguments: PathArguments::None,
	});

	syn::Expr::Path(ExprPath {
		attrs: Vec::new(),
		qself: None,
		path: Path {
			leading_colon: None,
			segments,
		},
	})
}

pub struct AdditionalGenerics {
	map: HashMap<Ident, Ident>,
	params: Vec<TypeParam>,
	other_ty_generics: Punctuated<GenericArgument, Comma>,
}

impl AdditionalGenerics {
	pub(crate) fn new(generics: &Generics, configs: &HashMap<Ident, ParamConfig>) -> Self {
		let mut map = HashMap::new();
		let mut params = Vec::new();
		let mut other_ty_generics = Punctuated::new();

		for p in generics.params.iter() {
			let other_arg = match p {
				syn::GenericParam::Type(ty) => {
					let conf = configs.get(&ty.ident).cloned().unwrap_or_default();

					let ident = if conf.fixed {
						map.insert(ty.ident.clone(), ty.ident.clone());
						ty.ident.clone()
					} else {
						let ident = Ident::new(&format!("__{}", ty.ident), ty.ident.span());
						map.insert(ty.ident.clone(), ident.clone());
						params.push(TypeParam {
							attrs: ty.attrs.clone(),
							ident: ident.clone(),
							colon_token: ty.colon_token,
							bounds: ty.bounds.clone(), // bounds depending on other params are not supported.
							eq_token: ty.eq_token,
							default: ty.default.clone(),
						});
						ident
					};

					GenericArgument::Type(type_ref(ident))
				}
				syn::GenericParam::Lifetime(l) => GenericArgument::Lifetime(l.lifetime.clone()),
				syn::GenericParam::Const(c) => GenericArgument::Const(const_ref(c.ident.clone())),
			};

			other_ty_generics.push_value(other_arg);
			other_ty_generics.push_punct(Comma::default());
		}

		Self {
			map,
			params,
			other_ty_generics,
		}
	}

	pub fn other_ty_generics(&self) -> syn::PathArguments {
		syn::PathArguments::AngleBracketed(AngleBracketedGenericArguments {
			colon2_token: None,
			lt_token: syn::token::Lt::default(),
			args: self.other_ty_generics.clone(),
			gt_token: syn::token::Gt::default(),
		})
	}

	pub fn get(&self, ident: &Ident) -> Option<&Ident> {
		self.map.get(ident)
	}

	pub fn get_as_args(&self, ident: &Ident) -> Option<PathArguments> {
		self.get(ident).map(|i| {
			PathArguments::AngleBracketed(AngleBracketedGenericArguments {
				colon2_token: None,
				lt_token: syn::token::Lt::default(),
				args: [GenericArgument::Type(type_ref(i.clone()))]
					.into_iter()
					.collect(),
				gt_token: syn::token::Gt::default(),
			})
		})
	}
}

pub struct TokensOrDefault<'a, T: 'a>(pub &'a Option<T>);

impl<'a, T> ToTokens for TokensOrDefault<'a, T>
where
	T: ToTokens + Default,
{
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self.0 {
			Some(t) => t.to_tokens(tokens),
			None => T::default().to_tokens(tokens),
		}
	}
}

pub struct ImplGenericsWithAdditional<'a>(pub &'a Generics, pub &'a AdditionalGenerics);

impl<'a> ToTokens for ImplGenericsWithAdditional<'a> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		if self.0.params.is_empty() {
			return;
		}

		TokensOrDefault(&self.0.lt_token).to_tokens(tokens);

		// Print lifetimes before types and consts, regardless of their
		// order in self.params.
		//
		// TODO: ordering rules for const parameters vs type parameters have
		// not been settled yet. https://github.com/rust-lang/rust/issues/44580
		let mut trailing_or_empty = true;
		for param in self.0.params.pairs() {
			if let GenericParam::Lifetime(_) = **param.value() {
				if !trailing_or_empty {
					<syn::Token![,]>::default().to_tokens(tokens);
				}

				param.to_tokens(tokens);
				trailing_or_empty = param.punct().is_some();
			}
		}

		for param in self.0.params.pairs() {
			if let GenericParam::Lifetime(_) = **param.value() {
				continue;
			}

			if !trailing_or_empty {
				<syn::Token![,]>::default().to_tokens(tokens);
			}
			match *param.value() {
				GenericParam::Lifetime(_) => unreachable!(),
				GenericParam::Type(param) => {
					// Leave off the type parameter defaults
					tokens.append_all(param.attrs.outer());
					param.ident.to_tokens(tokens);
					if !param.bounds.is_empty() {
						TokensOrDefault(&param.colon_token).to_tokens(tokens);
						param.bounds.to_tokens(tokens);
					}
				}
				GenericParam::Const(param) => {
					// Leave off the const parameter defaults
					tokens.append_all(param.attrs.outer());
					param.const_token.to_tokens(tokens);
					param.ident.to_tokens(tokens);
					param.colon_token.to_tokens(tokens);
					param.ty.to_tokens(tokens);
				}
			}
			param.punct().to_tokens(tokens);
			trailing_or_empty = param.punct().is_some();
		}

		for param in &self.1.params {
			if !trailing_or_empty {
				<syn::Token![,]>::default().to_tokens(tokens);
			}

			// Leave off the type parameter defaults
			tokens.append_all(param.attrs.outer());
			param.ident.to_tokens(tokens);
			if !param.bounds.is_empty() {
				TokensOrDefault(&param.colon_token).to_tokens(tokens);
				param.bounds.to_tokens(tokens);
			}

			trailing_or_empty = false
		}

		TokensOrDefault(&self.0.gt_token).to_tokens(tokens);
	}
}

// pub struct OtherTypeGenerics<'a>(&'a AdditionalGenerics);

// impl<'a> ToTokens for OtherTypeGenerics<'a> {
// 	fn to_tokens(&self, tokens: &mut TokenStream) {
// 		if self.0.other_ty_generics.is_empty() {
// 			return;
// 		}

// 		<syn::Token![<]>::default().to_tokens(tokens);

// 		for (i, param) in self.0.other_ty_generics.iter().enumerate() {
// 			if i > 0 {
// 				<syn::Token![,]>::default().to_tokens(tokens);
// 			}

// 			param.to_tokens(tokens);
// 		}

// 		<syn::Token![>]>::default().to_tokens(tokens);
// 	}
// }

pub trait FilterAttrs<'a> {
	type Ret: Iterator<Item = &'a Attribute>;

	fn outer(self) -> Self::Ret;
	fn inner(self) -> Self::Ret;
}

impl<'a> FilterAttrs<'a> for &'a [Attribute] {
	type Ret = iter::Filter<slice::Iter<'a, Attribute>, fn(&&Attribute) -> bool>;

	fn outer(self) -> Self::Ret {
		fn is_outer(attr: &&Attribute) -> bool {
			match attr.style {
				AttrStyle::Outer => true,
				AttrStyle::Inner(_) => false,
			}
		}
		self.iter().filter(is_outer)
	}

	fn inner(self) -> Self::Ret {
		fn is_inner(attr: &&Attribute) -> bool {
			match attr.style {
				AttrStyle::Inner(_) => true,
				AttrStyle::Outer => false,
			}
		}
		self.iter().filter(is_inner)
	}
}
