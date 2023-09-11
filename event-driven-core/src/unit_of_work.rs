use std::sync::Arc;

use crate::{
	outbox::IOutBox,
	prelude::{AtomicContextManager, BaseError, TRepository},
};

use async_trait::async_trait;
use tokio::sync::RwLock;

#[async_trait]
pub trait Executor {
	async fn new() -> Arc<RwLock<Self>>;
	async fn begin(&mut self) -> Result<(), BaseError>;
	async fn commit(&mut self) -> Result<(), BaseError>;
	async fn rollback(&mut self) -> Result<(), BaseError>;
}

#[derive(Clone)]
pub struct UnitOfWork<R, E>
where
	R: TRepository<E>,
	E: Executor,
{
	executor: Arc<RwLock<E>>,
	context: AtomicContextManager,
	pub repository: R,
}

impl<R, E> UnitOfWork<R, E>
where
	R: TRepository<E>,
	E: Executor,
{
	/// Creating Uow means to begin transaction.
	pub async fn new(context: AtomicContextManager) -> Self {
		let executor: Arc<RwLock<E>> = E::new().await;

		let mut uow = Self {
			repository: R::new(executor.clone()),
			context,
			executor,
		};
		uow.begin().await.unwrap();
		uow
	}

	pub fn switch_repository<DR: TRepository<E>>(mut self) -> UnitOfWork<DR, E> {
		let mut repo = DR::new(self.executor.clone());
		repo.set_events(self.repository().get_events());

		UnitOfWork {
			executor: self.executor.clone(),
			context: self.context,
			repository: repo,
		}
	}

	pub fn repository(&mut self) -> &mut R {
		&mut self.repository
	}
	pub async fn begin(&mut self) -> Result<(), BaseError> {
		let mut executor = self.executor.write().await;
		executor.begin().await
	}

	pub fn executor(&self) -> Arc<RwLock<E>> {
		self.executor.clone()
	}

	pub async fn commit<O: IOutBox<E>>(mut self) -> Result<(), BaseError> {
		// To drop uow itself!

		self._commit_hook::<O>().await?;

		self._commit().await
	}
	async fn _commit(&mut self) -> Result<(), BaseError> {
		let mut executor = self.executor.write().await;

		executor.commit().await
	}

	pub async fn rollback(self) -> Result<(), BaseError> {
		let mut executor = self.executor.write().await;
		executor.rollback().await
	}

	/// commit_hook is invoked right before the calling for commit
	/// which sorts out and processes outboxes and internally processable events.
	pub async fn _commit_hook<O: IOutBox<E>>(&mut self) -> Result<(), BaseError> {
		let event_sender = &mut self.context.write().await.sender;
		let mut outboxes = vec![];

		for e in self.repository.get_events() {
			if e.externally_notifiable() {
				outboxes.push(e.outbox());
			};
			if e.internally_notifiable() {
				event_sender.send(e.message_clone()).await.expect("Event Collecting failed!")
			}
		}
		O::add(self.executor(), outboxes).await
	}
}
