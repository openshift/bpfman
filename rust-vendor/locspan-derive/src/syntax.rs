use crate::{Method, ParamConfig};
use std::collections::HashMap;
use syn::parse;
use syn::punctuated::Punctuated;
use syn::token;

pub(crate) fn parse_field_attributes(attrs: &[syn::Attribute]) -> syn::Result<Method> {
	for attr in attrs {
		if attr.path.is_ident("locspan") {
			let tokens = attr.tokens.clone().into();
			let locspan_attr: FieldAttribute = syn::parse(tokens)?;

			return Ok(locspan_attr.method);
		}
	}

	Ok(Method::Normal)
}

struct FieldAttribute {
	_paren: token::Paren,
	method: Method,
	_method_ident: syn::Ident,
}

impl parse::Parse for FieldAttribute {
	fn parse(input: parse::ParseStream) -> syn::Result<Self> {
		let content;
		let _paren = syn::parenthesized!(content in input);
		let _method_ident = syn::Ident::parse(&content)?;

		let method = if _method_ident == "stripped" {
			Method::Stripped
		} else if _method_ident == "ignore" {
			Method::Ignore
		} else if _method_ident == "deref_stripped" {
			Method::DerefThenStripped
		} else if _method_ident == "deref2_stripped" {
			Method::Deref2ThenStripped
		} else if _method_ident == "unwrap_stripped" {
			Method::UnwrapThenStripped
		} else if _method_ident == "unwrap_deref_stripped" {
			Method::UnwrapThenDerefThenStripped
		} else if _method_ident == "unwrap_deref2_stripped" {
			Method::UnwrapThenDeref2ThenStripped
		} else {
			return Err(syn::Error::new_spanned(
				_method_ident,
				"invalid locspan-derive attribute",
			));
		};

		Ok(Self {
			_paren,
			method,
			_method_ident,
		})
	}
}

pub(crate) fn parse_type_attributes(
	attrs: Vec<syn::Attribute>,
) -> HashMap<proc_macro2::Ident, ParamConfig> {
	let mut params = HashMap::new();

	for attr in attrs {
		if attr.path.is_ident("locspan") {
			let tokens = attr.tokens.into();
			let locspan_attrs: TypeAttributes = syn::parse(tokens).unwrap();

			for ty_attr in locspan_attrs.list {
				for ident in ty_attr.idents {
					let conf: &mut ParamConfig = params.entry(ident).or_default();
					match ty_attr.kind {
						TypeAttributeKind::Ignore => conf.ignore = true,
						TypeAttributeKind::Stripped => conf.stripped = true,
						TypeAttributeKind::Fixed => conf.fixed = true,
					}
				}
			}
		}
	}

	params
}

struct TypeAttributes {
	_paren: token::Paren,
	list: Punctuated<TypeAttribute, token::Comma>,
}

impl parse::Parse for TypeAttributes {
	fn parse(input: parse::ParseStream) -> syn::Result<Self> {
		let content;
		Ok(Self {
			_paren: syn::parenthesized!(content in input),
			list: Punctuated::parse_terminated(&content)?,
		})
	}
}

pub enum TypeAttributeKind {
	Ignore,
	Stripped,
	Fixed,
}

pub struct TypeAttribute {
	kind: TypeAttributeKind,
	_kind_ident: syn::Ident,
	_paren: token::Paren,
	idents: Punctuated<syn::Ident, token::Comma>,
}

impl parse::Parse for TypeAttribute {
	fn parse(input: parse::ParseStream) -> syn::Result<Self> {
		let kind_ident = syn::Ident::parse(input)?;

		let kind = if kind_ident == "ignore" {
			TypeAttributeKind::Ignore
		} else if kind_ident == "stripped" {
			TypeAttributeKind::Stripped
		} else if kind_ident == "fixed" {
			TypeAttributeKind::Fixed
		} else {
			return Err(syn::Error::new_spanned(
				kind_ident,
				"invalid locspan-derive attribute",
			));
		};

		let content;
		Ok(Self {
			kind,
			_kind_ident: kind_ident,
			_paren: syn::parenthesized!(content in input),
			idents: Punctuated::parse_terminated(&content)?,
		})
	}
}
