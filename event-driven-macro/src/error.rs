use proc_macro::TokenStream;
use syn::DeriveInput;

pub(crate) fn render_error_token(ast: &DeriveInput) -> TokenStream {
	// Forcing target to be enum
	match ast.data {
		syn::Data::Enum(_) => {}
		_ => {
			panic!("#[derive(ApplicationError)] is only support enum.")
		}
	}

	let name = &ast.ident;
	let crates = ast.attrs.iter().find(|x| x.path().is_ident("crates"));
	let crates = if let Some(crates) = crates {
		crates.parse_args::<syn::ExprPath>().unwrap().path.get_ident().expect("#[crates(...)] expects path.").to_string()
	} else {
		"event_driven_library".to_owned()
	};
	let crates = syn::Ident::new(&crates, proc_macro2::Span::call_site());
	let error_with_event = ast.attrs.iter().find(|x| x.path().is_ident("error_with_event"));
	let database_error = ast.attrs.iter().find(|x| x.path().is_ident("database_error"));
	let service_error = ast.attrs.iter().find(|x| x.path().is_ident("service_error"));

	quote!(
		impl ::std::error::Error for #name {}
		impl #crates::event_driven_core::responses::ApplicationError for #name {}
		impl ::std::convert::From<#crates::event_driven_core::responses::BaseError> for #name {
			fn from(value: #crates::event_driven_core::responses::BaseError) -> Self {
				// #name::BaseError(value)
				todo!()
			}
		}
		impl ::std::convert::Into<::std::boxed::Box<dyn #crates::event_driven_core::responses::ApplicationError>> for #name {
			fn into(self) -> ::std::boxed::Box<dyn #crates::event_driven_core::responses::ApplicationError> {
				::std::boxed::Box::new(self)
			}
		}
	)
	.into()
}
