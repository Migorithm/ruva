use proc_macro::TokenStream;
use syn::DeriveInput;

pub fn render_repository_token(ast: &DeriveInput) -> TokenStream {
	let name = &ast.ident;

	quote!(
		impl<A: TAggregate + 'static> #name<A> {
			fn event_hook(
				&mut self,
				aggregate: &mut A,
			) {
				self.set_events(aggregate.take_events());
			}
		}
	)
	.into()
}

pub fn render_event_hook(ast: &DeriveInput) -> TokenStream {
	let name = &ast.ident;

	quote!(
		#[ruva::prelude::async_trait]
		impl<A: TAggregate + 'static> ::ruva::ruva_core::unit_of_work::TCommitHook for #name<A> {}
	)
	.into()
}
