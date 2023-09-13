use proc_macro::TokenStream;
use syn::DeriveInput;

pub(crate) fn render_error_token(ast: &DeriveInput) -> TokenStream {
	// Forcing target to be enum
	let data = match &ast.data {
		syn::Data::Enum(data) => data,
		_ => {
			panic!("#[derive(ApplicationError)] is only support enum.")
		}
	};

	let name = &ast.ident;
	let find_variant = |name: &str| data.variants.iter().find(|x| x.attrs.iter().find(|x| x.path().is_ident(name)).is_some());

	/* \#\[crates(...)\] */
	let crates = ast.attrs.iter().find(|x| x.path().is_ident("crates"));
	let crates = if let Some(crates) = crates {
		crates.parse_args::<syn::ExprPath>().unwrap().path.get_ident().expect("#[crates(...)] expects path.").to_string()
	} else {
		"event_driven_library".to_owned()
	};
	let crates = syn::Ident::new(&crates, proc_macro2::Span::call_site());

	/* \#\[error\] */
	let error = find_variant("error");
	if let Some(error) = error {
		if let syn::Fields::Unit = error.fields {
		} else {
			panic!("#[error] expects unit.")
		}
	}
	let error = if let Some(error) = error {
		error.ident.clone()
	} else {
		syn::Ident::new("StopSentinel", proc_macro2::Span::call_site())
	};

	/* \#\[error_with_event\] */
	let error_with_event = find_variant("error_with_event");
	if let Some(error_with_event) = error_with_event {
		if let syn::Fields::Unnamed(_) = error_with_event.fields {
		} else {
			panic!("#[error_with_event] expects Field(Box<AnyError>).")
		}
	}
	let error_with_event = if let Some(error_with_event) = error_with_event {
		error_with_event.ident.clone()
	} else {
		syn::Ident::new("StopSentinelWithEvent", proc_macro2::Span::call_site())
	};

	/* \#\[database_error\] */
	let database_error = find_variant("database_error");
	if let Some(database_error) = database_error {
		if let syn::Fields::Unnamed(_) = database_error.fields {
		} else {
			panic!("#[database_error] expects Field(Box<AnyError>).")
		}
	}
	let database_error = if let Some(database_error) = database_error {
		database_error.ident.clone()
	} else {
		syn::Ident::new("DatabaseError", proc_macro2::Span::call_site())
	};

	quote!(
		impl #crates::event_driven_core::responses::ApplicationError for #name {}
		impl ::std::convert::From<#crates::event_driven_core::responses::BaseError> for #name {
			fn from(value: #crates::event_driven_core::responses::BaseError) -> Self {
				match value {
					#crates::event_driven_core::responses::BaseError::StopSentinel => Self::#error,
					#crates::event_driven_core::responses::BaseError::StopSentinelWithEvent(event) => Self::#error_with_event(event),
					#crates::event_driven_core::responses::BaseError::DatabaseError(error) => Self::#database_error(error),
					_ => unimplemented!("BaseError to #name is only support for StopSentinel, StopSentinelWithEvent, DatabaseError."),
				}
			}
		}
		impl ::std::convert::Into<::std::boxed::Box<dyn  #crates::event_driven_core::responses::BaseError>> for #name {
			fn into(self) -> ::std::boxed::Box<dyn  #crates::event_driven_core::responses::BaseError> {
				let data = match self {
					#name::#error => #crates::event_driven_core::responses::BaseError::StopSentinel,
					#name::#error_with_event(event) => #crates::event_driven_core::responses::BaseError::StopSentinelWithEvent(event),
					#name::#database_error(error) => #crates::event_driven_core::responses::BaseError::DatabaseError(error),
					_ => #crates::event_driven_core::responses::BaseError::ServiceError(error),
				};
				::std::boxed::Box::new(data)
			}
		}
	)
	.into()
}
