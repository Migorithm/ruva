use proc_macro2::TokenStream;
use syn::{parse_quote, Data, DataStruct, DeriveInput, Fields, FieldsNamed, FnArg, ImplItemFn, ItemFn, Meta, Pat, PatIdent, PatType, Path, ReturnType, Signature, Stmt};

use crate::utils::{get_attributes, locate_crate_on_derive_macro};

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

///
/// #[aggregate]
/// #[derive(Default, Serialize, Deserialize)]
/// struct MyAggregate {
///     #[identifier]
///     pub age: i64,
/// }
///
/// #[async_trait]
/// impl TRepository<TExecutor, MyAggregate> for SqlRepository<MyAggregate> {
///     fn new(executor: Arc<RwLock<TExecutor>>) -> Self {
///          todo!()
///     }
///     async fn get(
///         &self,
///         aggregate_id: i64,
///     ) -> Result<MyAggregate, BaseError> {
///         todo!()
///     }
///
///     #[event_hook]
///     async fn update(
///         &mut self,
///         aggregate: &mut MyAggregate,
///     ) -> Result<(), BaseError> {
///         Ok(())
///     }
///     async fn add(
///         &mut self,
///         aggregate: &mut MyAggregate,
///     ) -> Result<i64, BaseError> {
///         todo!()
///     }
///     async fn delete(
///         &self,
///         _aggregate_id: i64,
///     ) -> Result<(), BaseError> {
///         todo!()
///     }
/// }
pub(crate) fn event_hook(mut ast: ItemFn) -> TokenStream {
	if ast.sig.inputs.is_empty() {
		panic!("There must be message argument!");
	};

	let aggregate = &ast.sig.inputs[1];

	if let FnArg::Typed(PatType { pat, ty, .. }) = aggregate.clone() {
		if let Pat::Ident(PatIdent { ident, .. }) = *pat.clone() {
			let mut stmts = vec![parse_quote!(
				self.event_hook(#ident);
			)];
			stmts.extend(std::mem::take(&mut ast.block.stmts));

			ast.block.stmts = stmts;
			return quote!(
				#ast
			);
		}
	}
	panic!("Not Processable!")
}
