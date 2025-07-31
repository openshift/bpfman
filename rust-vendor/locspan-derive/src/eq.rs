use crate::util;
use proc_macro2::TokenStream;
use quote::quote;

pub fn derive(input: syn::DeriveInput) -> TokenStream {
	let ident = input.ident;
	let mut generics = input.generics;
	let params = crate::syntax::parse_type_attributes(input.attrs);

	for p in generics.params.iter_mut() {
		if let syn::GenericParam::Type(ty) = p {
			let conf = params.get(&ty.ident).cloned().unwrap_or_default();
			if !conf.ignore {
				let path = if conf.stripped {
					util::simple_path(["core", "cmp"], "Eq", syn::PathArguments::None)
				} else {
					util::simple_path(["locspan"], "StrippedEq", syn::PathArguments::None)
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

	quote! {
		impl #impl_generics ::locspan::StrippedEq for #ident #ty_generics #where_clause {}
	}
}
