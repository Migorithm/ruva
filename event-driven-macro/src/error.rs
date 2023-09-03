use proc_macro::TokenStream;
use syn::DeriveInput;

pub(crate) fn render_error_token(ast: &DeriveInput) -> TokenStream {
	let name = &ast.ident;

	quote!(

		impl std::error::Error for #name {}
		impl ApplicationError for #name {}
		impl From<BaseError> for #name {
			fn from(value: BaseError) -> Self {
				#name::BaseError(value)
			}
		}
		impl From<std::boxed::Box<#name>> for std::boxed::Box<dyn ApplicationError> {
			fn from(value: std::boxed::Box<#name>) -> Self {
				value
			}
		}
		impl From<#name> for std::boxed::Box<dyn ApplicationError> {
			fn from(value: #name) -> Self {
				std::boxed::Box::new(value)
			}
		}
	)
	.into()
}
