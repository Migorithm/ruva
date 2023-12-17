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
