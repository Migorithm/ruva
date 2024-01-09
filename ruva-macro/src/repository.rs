use proc_macro::TokenStream;
use syn::DeriveInput;

pub fn render_repository_token(ast: &DeriveInput) -> TokenStream {
	let name = &ast.ident;

	quote!(
		impl<A: ruva::ruva_core::aggregate::TAggregate> #name<A> {
			pub fn event_hook(
				&mut self,
				aggregate: &mut A,
			) {
				self.0.set_events(aggregate.take_events());
			}
		}

		impl<A: ruva::ruva_core::aggregate::TAggregate> #name<A> {
			pub fn new(
				context: AtomicContextManager,
				executor: Arc<RwLock<SQLExecutor>>,
			) -> Self {
				Self(SqlRepository::new(context, executor))
			}
		}

		impl<A: ruva::ruva_core::aggregate::TAggregate> ruva::ruva_core::repository::TRepository for #name<A> {
			fn set_events(
				&mut self,
				events: std::collections::VecDeque<std::sync::Arc<dyn TEvent>>,
			) {
				self.0.set_events(events)
			}
		}

		impl<A: ruva::ruva_core::aggregate::TAggregate> ::std::ops::Deref for #name<A> {
			type Target = ruva::ruva_core::rdb::repository::SqlRepository<A>;
			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}
		impl<A: ruva::ruva_core::aggregate::TAggregate> ::std::ops::DerefMut for #name<A> {
			fn deref_mut(&mut self) -> &mut Self::Target {
				&mut self.0
			}
		}



		impl<A: ruva::ruva_core::aggregate::TAggregate> ruva::ruva_core::unit_of_work::TUnitOfWork for #name<A> {
			async fn begin(&mut self) -> Result<(), ruva::ruva_core::responses::BaseError> {
				self.0.begin().await
			}

			async fn commit(&mut self) -> Result<(), ruva::ruva_core::responses::BaseError> {
				self.0.commit().await
			}

			async fn rollback(&mut self) -> Result<(), ruva::ruva_core::responses::BaseError> {
				self.0.rollback().await
			}

			async fn close(&self) {
				self.0.close().await;
			}
		}


		impl<A: ruva::ruva_core::aggregate::TAggregate> ruva::ruva_core::unit_of_work::TCommitHook for #name<A> {
			async fn commit_hook(&mut self) -> Result<(), ruva::ruva_core::responses::BaseError> {
				self.0.commit_hook().await
			}
		}


		impl<A: ruva::ruva_core::aggregate::TAggregate> ruva::ruva_core::utils::TClone for #name<A> {
			fn clone(&self) -> Self {
				Self(self.0.clone())
			}
		}
		impl<A: ruva::ruva_core::aggregate::TAggregate> ruva::ruva_core::utils::TCloneContext for #name<A> {
			fn clone_context(&self) -> ruva::ruva_core::messagebus::AtomicContextManager {
				self.0.clone_context()
			}
		}


	)
	.into()
}
