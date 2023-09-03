use proc_macro::TokenStream;
use syn::DeriveInput;

pub(crate) fn render_aggregate_token(ast: &DeriveInput) -> TokenStream {
	let name = &ast.ident;

	quote!(
		impl Aggregate for #name{

			fn events(&self) -> &std::collections::VecDeque<Box<dyn Message>> {
				&self.events
			}
			fn take_events(&mut self) -> std::collections::VecDeque<Box<dyn Message>> {
				std::mem::take(&mut self.events)
			}
			fn raise_event(&mut self, event: Box<dyn Message>) {
				self.events.push_back(event)
			}

		}
	)
	.into()
}
