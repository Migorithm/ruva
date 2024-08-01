use quote::ToTokens;
use syn::{punctuated::Punctuated, token::Plus, Generics, TypeParamBound, WherePredicate};

// Add `Send`, `Sync`, `'static` and `std::fmt::Debug` to TypeGenerics if it doesn't have them
// If they have, then selectively add `Send`, `Sync`, `'static` and `std::fmt::Debug` to TypeGenerics
// Plus, move T type constraints to where clause
pub fn add_sync_trait_bounds(generics: &mut syn::Generics) {
	generics
		.params
		.iter_mut()
		.filter_map(|param| match param {
			syn::GenericParam::Type(ty) => Some(ty),
			_ => None,
		})
		.for_each(|ty| {
			// optimize the above code
			add_type_param_bounds(&mut ty.bounds, &["Send", "Sync", "'static", "std::fmt::Debug"]);
		})
}

/// add `Send`, `Sync`, `'static`, `std::fmt::Debug` and `Default` to TypeGenerics if it doesn't have them
/// The following will move contraints from TypeGenerics to where clause
pub fn add_aggregate_generic_defaults_on_where_clause(generics: &mut Generics) {
	let mut predicates: Punctuated<WherePredicate, Plus> = Punctuated::new();
	generics.params.iter().for_each(|param| {
		if let syn::GenericParam::Type(ty) = param {
			// get syn::Type from syn::TypeParam
			let ty_path = syn::TypePath {
				qself: None,
				path: ty.ident.clone().into(),
			};
			let ty = syn::Type::Path(ty_path);

			predicates.push(WherePredicate::Type(syn::PredicateType {
				bounded_ty: ty,
				colon_token: Default::default(),
				bounds: Default::default(),
				lifetimes: None,
			}));
		}
	});

	predicates.iter_mut().for_each(|predicate| {
		if let syn::WherePredicate::Type(predicate) = predicate {
			add_type_param_bounds(&mut predicate.bounds, &["Send", "Sync", "'static", "std::fmt::Debug", "Default"])
		}
	});

	if !predicates.is_empty() {
		if let Some(where_clause) = generics.where_clause.as_mut() {
			where_clause.predicates.extend(predicates);
		} else {
			generics.make_where_clause().predicates.extend(predicates);
		};
	}
}

fn add_type_param_bounds<T: Default>(predicate_bounds: &mut Punctuated<TypeParamBound, T>, contraints: &[&str]) {
	let bounds = predicate_bounds.iter().map(|b| b.to_token_stream().to_string()).collect::<Vec<_>>();

	for contraint in contraints {
		if bounds.contains(&contraint.to_string()) {
			continue;
		}
		match *contraint {
			"Send" => predicate_bounds.push(syn::parse_quote!(Send)),
			"Sync" => predicate_bounds.push(syn::parse_quote!(Sync)),
			"'static" => predicate_bounds.push(syn::parse_quote!('static)),
			"std::fmt::Debug" => predicate_bounds.push(syn::parse_quote!(std::fmt::Debug)),
			"Default" => predicate_bounds.push(syn::parse_quote!(Default)),
			"Clone" => predicate_bounds.push(syn::parse_quote!(Clone)),
			"Serialize" => predicate_bounds.push(syn::parse_quote!(serde::Serialize)),
			"Deserialize" => predicate_bounds.push(syn::parse_quote!(serde::Deserialize)),
			_ => {}
		}
	}
}
