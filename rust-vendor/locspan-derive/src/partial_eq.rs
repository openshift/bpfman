use crate::{
	util::{self, AdditionalGenerics, TryCollect},
	Access, Method,
};
use proc_macro2::TokenStream;
use quote::quote;

pub enum Error {
	Syntax(syn::Error),
	Union,
}

pub fn derive(input: syn::DeriveInput) -> Result<TokenStream, Error> {
	let ident = input.ident;
	let mut generics = input.generics;
	let params = crate::syntax::parse_type_attributes(input.attrs);

	let additional_generics = AdditionalGenerics::new(&generics, &params);

	for p in generics.params.iter_mut() {
		if let syn::GenericParam::Type(ty) = p {
			let conf = params.get(&ty.ident).cloned().unwrap_or_default();

			if !conf.ignore {
				let path = if conf.stripped {
					util::simple_path(
						["core", "cmp"],
						"PartialEq",
						additional_generics.get_as_args(&ty.ident).unwrap(),
					)
				} else {
					util::simple_path(
						["locspan"],
						"StrippedPartialEq",
						additional_generics.get_as_args(&ty.ident).unwrap(),
					)
				};

				ty.bounds.push(syn::TypeParamBound::Trait(syn::TraitBound {
					paren_token: None,
					modifier: syn::TraitBoundModifier::None,
					lifetimes: None,
					path,
				}));
			}
		}
	}

	let (_, ty_generics, where_clause) = generics.split_for_impl();
	let impl_generics = crate::util::ImplGenericsWithAdditional(&generics, &additional_generics);
	let other_ty_generics = additional_generics.other_ty_generics();

	let comparison = match input.data {
		syn::Data::Struct(s) => {
			fields_comparisons(crate::fields_access_pairs(&s.fields)).map_err(Error::Syntax)?
		}
		syn::Data::Enum(e) => {
			let cases = e
				.variants
				.iter()
				.map(|v| {
					let (self_pattern, self_args) = crate::variant_pattern(v, |arg| {
						Some(syn::Ident::new(
							&format!("__self_{}", arg),
							proc_macro2::Span::call_site(),
						))
					});

					let (other_pattern, other_args) = crate::variant_pattern(v, |arg| {
						Some(syn::Ident::new(
							&format!("__other_{}", arg),
							proc_macro2::Span::call_site(),
						))
					});

					let comparisons = fields_comparisons(
						v.fields.iter().zip(
							self_args
								.into_iter()
								.map(Access::Reference)
								.zip(other_args.into_iter().map(Access::Reference)),
						),
					)
					.map_err(Error::Syntax)?;

					Ok(quote! {
						(Self::#self_pattern, #ident::#other_pattern) => {
							#comparisons
						}
					})
				})
				.try_collect()?;

			quote! {
				match (self, other) {
					#(#cases),*
					_ => false
				}
			}
		}
		syn::Data::Union(_) => return Err(Error::Union),
	};

	Ok(quote! {
		impl #impl_generics ::locspan::StrippedPartialEq<#ident #other_ty_generics> for #ident #ty_generics #where_clause {
			fn stripped_eq(&self, other: &#ident #other_ty_generics) -> bool {
				#comparison
			}
		}
	})
}

fn fields_comparisons<'a>(
	fields: impl 'a + IntoIterator<Item = (&'a syn::Field, (Access, Access))>,
) -> syn::Result<proc_macro2::TokenStream> {
	let mut comparisons = proc_macro2::TokenStream::new();

	for (field, (self_path, other_path)) in fields {
		if let Some(c) = field_comparisons(field, self_path, other_path)? {
			if !comparisons.is_empty() {
				comparisons.extend(quote! { && })
			}

			comparisons.extend(c)
		}
	}

	if comparisons.is_empty() {
		comparisons.extend(quote! { true })
	}

	Ok(comparisons)
}

fn field_comparisons(
	field: &syn::Field,
	self_path: Access,
	other_path: Access,
) -> syn::Result<Option<proc_macro2::TokenStream>> {
	let method = crate::syntax::parse_field_attributes(&field.attrs)?;

	Ok(match method {
		Method::Ignore => None,
		Method::Normal => {
			let self_path = self_path.by_ref();
			let other_path = other_path.by_ref();
			Some(quote! { ::locspan::StrippedPartialEq::stripped_eq(#self_path, #other_path) })
		}
		Method::Stripped => Some(quote! { #self_path == #other_path }),
		Method::DerefThenStripped => {
			let self_path = self_path.by_deref();
			let other_path = other_path.by_deref();
			Some(quote! { #self_path == #other_path })
		}
		Method::Deref2ThenStripped => {
			let self_path = self_path.by_deref();
			let other_path = other_path.by_deref();
			Some(quote! { *#self_path == *#other_path })
		}
		Method::UnwrapThenStripped => Some(
			quote! { #self_path.as_ref().zip(#other_path.as_ref()).map(|(a, b)| *a == *b).unwrap_or(true) },
		),
		Method::UnwrapThenDerefThenStripped => Some(
			quote! { #self_path.as_ref().zip(#other_path.as_ref()).map(|(a, b)| **a == **b).unwrap_or(true) },
		),
		Method::UnwrapThenDeref2ThenStripped => Some(
			quote! { #self_path.as_ref().zip(#other_path.as_ref()).map(|(a, b)| ***a == ***b).unwrap_or(true) },
		),
	})
}
