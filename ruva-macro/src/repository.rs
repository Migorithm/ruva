use proc_macro::TokenStream;
use syn::DeriveInput;

pub fn render_repository_token(ast: &DeriveInput) -> TokenStream {
	let name = &ast.ident;

	quote!(

		impl #name {
			pub fn event_hook<A:TAggregate>(
				&mut self,
				aggregate: &mut A,
			) {
				self.0.set_events(aggregate.take_events());
			}
		}



		impl ruva::ruva_core::repository::TRepository for #name {
			fn set_events(
				&mut self,
				events: std::collections::VecDeque<std::sync::Arc<dyn TEvent>>,
			) {
				self.0.set_events(events)
			}
		}

		impl ::std::ops::Deref for #name {
			type Target = ruva::ruva_core::rdb::repository::SqlRepository;
			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}
		impl ::std::ops::DerefMut for #name {
			fn deref_mut(&mut self) -> &mut Self::Target {
				&mut self.0
			}
		}



		impl ruva::ruva_core::unit_of_work::TUnitOfWork for #name {
			async fn begin(&mut self) -> Result<(), ruva::ruva_core::responses::BaseError> {
				self.0.begin().await
			}

			async fn _commit(&mut self) -> Result<(), ruva::ruva_core::responses::BaseError> {
				self.0._commit().await
			}

			async fn rollback(&mut self) -> Result<(), ruva::ruva_core::responses::BaseError> {
				self.0.rollback().await
			}

			async fn close(&mut self) {
				self.0.close().await;
			}
			async fn process_internal_events(&mut self) ->  Result<(), ruva::ruva_core::responses::BaseError> {
				self.0.process_internal_events().await
			}
			async fn process_external_events(&mut self) -> Result<(), ruva::ruva_core::responses::BaseError> {
				self.0.process_external_events().await
			}
		}




	)
	.into()
}
