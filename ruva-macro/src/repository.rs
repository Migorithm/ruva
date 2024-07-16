use proc_macro::TokenStream;
use syn::DeriveInput;

pub fn render_repository_token(ast: &DeriveInput) -> TokenStream {
	let name = &ast.ident;
	let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

	quote!(

		impl #impl_generics #ty_generics #name #where_clause {
			pub fn event_hook<A:TAggregate>(
				&mut self,
				aggregate: &mut A,
			) {
				self.0.set_events(aggregate.take_events());
			}
		}



		impl #impl_generics #ty_generics ruva::ruva_core::repository::TRepository for #name #impl_generics #where_clause {
			fn set_events(
				&mut self,
				events: std::collections::VecDeque<std::sync::Arc<dyn TEvent>>,
			) {
				self.0.set_events(events)
			}
		}

		impl #impl_generics #ty_generics ::std::ops::Deref for #name #impl_generics #ty_generics #where_clause {
			type Target = ruva::ruva_core::rdb::repository::SqlRepository;
			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}
		impl #impl_generics #ty_generics ::std::ops::DerefMut for #name #impl_generics #ty_generics #where_clause {
			fn deref_mut(&mut self) -> &mut Self::Target<'a> {
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
