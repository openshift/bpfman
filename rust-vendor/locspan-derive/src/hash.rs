use crate::{
	util::{self, TryCollect},
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

	for p in generics.params.iter_mut() {
		if let syn::GenericParam::Type(ty) = p {
			let conf = params.get(&ty.ident).cloned().unwrap_or_default();
			if !conf.ignore {
				let path = if conf.stripped {
					util::simple_path(["core", "hash"], "Hash", syn::PathArguments::None)
				} else {
					util::simple_path(["locspan"], "StrippedHash", syn::PathArguments::None)
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

	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let hash = match input.data {
		syn::Data::Struct(s) => {
			fields_hash(crate::fields_access(&s.fields)).map_err(Error::Syntax)?
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

					let hash = fields_hash(
						v.fields
							.iter()
							.zip(self_args.into_iter().map(Access::Reference)),
					)
					.map_err(Error::Syntax)?;

					Ok(quote! {
						Self::#self_pattern => {
							#hash
						}
					})
				})
				.try_collect()?;

			quote! {
				match self {
					#(#cases),*
				}
			}
		}
		syn::Data::Union(_) => return Err(Error::Union),
	};

	Ok(quote! {
		impl #impl_generics ::locspan::StrippedHash for #ident #ty_generics #where_clause {
			fn stripped_hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
				#hash
			}
		}
	})
}

fn fields_hash<'a>(
	fields: impl 'a + IntoIterator<Item = (&'a syn::Field, Access)>,
) -> syn::Result<proc_macro2::TokenStream> {
	let mut partial_ord = proc_macro2::TokenStream::new();

	for (field, path) in fields {
		if let Some(c) = field_hash(field, path)? {
			if !partial_ord.is_empty() {
				partial_ord.extend(quote! { ; });
			}

			partial_ord.extend(c)
		}
	}

	if partial_ord.is_empty() {
		partial_ord.extend(quote! { ::core::hash::Hash::hash(&0xffu32, state) })
	}

	Ok(partial_ord)
}

fn field_hash(field: &syn::Field, path: Access) -> syn::Result<Option<proc_macro2::TokenStream>> {
	let method = crate::syntax::parse_field_attributes(&field.attrs)?;

	Ok(match method {
		Method::Ignore => None,
		Method::Normal => {
			let path = path.by_ref();
			Some(quote! { ::locspan::StrippedHash::stripped_hash(#path, state) })
		}
		Method::Stripped => {
			let path = path.by_ref();
			Some(quote! { ::core::hash::Hash::hash(#path, state) })
		}
		Method::DerefThenStripped => {
			let path = path.by_deref();
			Some(quote! { ::core::hash::Hash::hash(&#path, state) })
		}
		Method::Deref2ThenStripped => {
			let path = path.by_deref();
			Some(quote! { ::core::hash::Hash::hash(&*#path, state) })
		}
		Method::UnwrapThenStripped => Some(quote! {
			match #path.as_ref() {
				Some(v) => ::core::hash::Hash::hash(&*v, state),
				None => ::core::hash::Hash::hash(&0xffu32, state)
			}
		}),
		Method::UnwrapThenDerefThenStripped => Some(quote! {
			match #path.as_ref() {
				Some(v) => ::core::hash::Hash::hash(&**v, state),
				None => ::core::hash::Hash::hash(&0xffu32, state)
			}
		}),
		Method::UnwrapThenDeref2ThenStripped => Some(quote! {
			match #path.as_ref() {
				Some(v) => ::core::hash::Hash::hash(&***v, state),
				None => ::core::hash::Hash::hash(&0xffu32, state)
			}
		}),
	})
}
