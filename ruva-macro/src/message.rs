use proc_macro2::TokenStream;
use syn::{parse_quote, Data, DataStruct, DeriveInput, Fields, FieldsNamed, FnArg, ItemFn, Meta, Pat, PatIdent, PatType, Path, Type};

use crate::utils::{get_attributes, get_trait_checking_stmts, locate_crate_on_derive_macro};

pub(crate) fn render_message_token(ast: &DeriveInput, propagatability: Vec<TokenStream>, identifier: TokenStream) -> TokenStream {
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

pub(crate) fn find_identifier(ast: &DeriveInput) -> TokenStream {
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

	let mut stmts = get_trait_checking_stmts("Aggregate");

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
