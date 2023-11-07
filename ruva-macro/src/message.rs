use proc_macro2::TokenStream;
use syn::{parse_quote, Data, DataStruct, DeriveInput, Fields, FieldsNamed, FnArg, ItemFn, Meta, MetaList, Pat, PatIdent, PatType, Path, Type};

use crate::utils::{get_attributes, get_trait_checking_stmts, locate_crate_on_derive_macro};

pub(crate) fn render_message_token(ast: &DeriveInput, propagatability: Vec<TokenStream>, identifier: TokenStream, impl_assertion: TokenStream) -> TokenStream {
	let name = &ast.ident;
	let crates = locate_crate_on_derive_macro(ast);

	quote! {
		impl #crates::prelude::Message for #name {

			#identifier

			fn message_clone(&self) -> ::std::boxed::Box<dyn #crates::prelude::Message> {
				::std::boxed::Box::new(self.clone())
			}
			fn state(&self) -> ::std::string::String {
				serde_json::to_string(&self).expect("Failed to serialize")
			}
			fn to_message(self)-> ::std::boxed::Box<dyn #crates::prelude::Message+'static>{
				::std::boxed::Box::new(self)
			}

			#(#propagatability)*
		}
		impl #crates::prelude::MailSendable for #name {
			fn template_name(&self) -> ::std::string::String {
				// * subject to change
				stringify!(#name).into()
			}
		}

		#impl_assertion
	}
}

pub(crate) fn render_event_visibility(ast: &DeriveInput) -> Vec<TokenStream> {
	let propagatability = ast
		.attrs
		.iter()
		.flat_map(|attr| {
			if let Meta::Path(Path { segments, .. }) = &attr.meta {
				segments
					.iter()
					.filter_map(|s| {
						if s.ident.to_string().as_str() == "internally_notifiable" {
							Some(quote!(
								fn internally_notifiable(&self) -> bool {
									true
								}
							))
						} else if s.ident.to_string().as_str() == "externally_notifiable" {
							Some(quote!(
								fn externally_notifiable(&self) -> bool {
									true
								}
							))
						} else {
							None
						}
					})
					.collect::<Vec<_>>()
			} else {
				panic!("Error!")
			}
		})
		.collect::<Vec<_>>();
	if propagatability.is_empty() {
		panic!("Either `internally_notifiable` or `externally_notifiable` must be given!")
	}
	propagatability
}

pub(crate) fn get_aggregate_metadata(ast: &mut DeriveInput) -> (TokenStream, String) {
	let mut idx = 10000;
	let mut aggregate_name: Option<String> = None;

	let res = ast
		.attrs
		.iter_mut()
		.enumerate()
		.flat_map(|(order, attr)| {
			if let Meta::List(MetaList { path, tokens, .. }) = &mut attr.meta {
				let ident = path.get_ident();
				if ident.unwrap() != "aggregate" {
					panic!("MetaList is allowed only for aggregate!");
				}
				let quote = quote!(
					ruva::static_assertions::assert_impl_any!(#tokens: ruva::prelude::Aggregate);
				);

				tokens.clone().into_iter().for_each(|t| {
					if let proc_macro2::TokenTree::Ident(ident) = t {
						aggregate_name = Some(ident.to_string());
					}
				});

				if aggregate_name.is_none() || aggregate_name.as_ref().unwrap().is_empty() {
					panic!("Aggregate name must be given!");
				}

				idx = order;

				Some((quote, aggregate_name.clone().unwrap()))
			} else {
				None
			}
		})
		.collect::<Vec<_>>();

	let res = res.first().expect("aggregate must be specified! \rExample: #[aggregate(CustomAggregate)]\n").clone();
	ast.attrs.remove(idx);

	res
}

pub(crate) fn find_identifier(ast: &DeriveInput, aggregate_metadata: String) -> TokenStream {
	let name = &ast.ident;
	let crates = locate_crate_on_derive_macro(ast);

	match &ast.data {
		Data::Struct(DataStruct {
			fields: Fields::Named(FieldsNamed { named, .. }),
			..
		}) => {
			let identifier = named.iter().filter(|f| get_attributes(f).into_iter().any(|ident| ident == *"identifier")).collect::<Vec<_>>();
			if identifier.len() != 1 {
				panic!("One identifier Must Be Given To Message!")
			}

			let ident = identifier.first().unwrap().ident.clone().unwrap().clone();

			quote!(
				fn metadata(&self) -> #crates::prelude::MessageMetadata {
					#crates::prelude::MessageMetadata{
					aggregate_id: self.#ident.to_string(),
					aggregate_name: #aggregate_metadata.into(),
					topic: stringify!(#name).into()
				}
			}
			)
		}
		_ => panic!("Only Struct Allowed!"),
	}
}

pub(crate) fn event_hook(mut ast: ItemFn) -> TokenStream {
	if ast.sig.inputs.is_empty() {
		panic!("There must be message argument!");
	};

	let mut stmts = get_trait_checking_stmts("::ruva::prelude::Aggregate");

	for aggregate in &ast.sig.inputs.iter().skip(1).collect::<Vec<_>>() {
		if let FnArg::Typed(PatType { pat, ty, .. }) = aggregate {
			let ty: Box<Type> = match *ty.clone() {
				Type::Reference(a) => a.elem,
				ty => Box::new(ty),
			};

			if let Pat::Ident(PatIdent { ident, .. }) = *pat.clone() {
				stmts.push(parse_quote!(
					if <IsTrait<#ty>>::IS_TRAIT {
						self.event_hook(<IsTrait<#ty>>::get_data(#ident));
					}
				));
			}
		}
	}
	stmts.extend(std::mem::take(&mut ast.block.stmts));
	ast.block.stmts = stmts;
	quote!(
		#ast
	)
}
