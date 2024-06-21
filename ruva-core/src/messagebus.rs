use crate::prelude::{TCommand, TCommandService, TEvent};
use crate::responses::{self, ApplicationError, ApplicationResponse, BaseError};
use async_trait::async_trait;
use hashbrown::HashMap;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::{pin::Pin, sync::Arc};
use tokio::sync::RwLock;

pub type Future<T, E> = Pin<Box<dyn futures::Future<Output = Result<T, E>> + Send>>;
pub type AtomicContextManager = Arc<RwLock<ContextManager>>;

pub type Handlers<R, E> = Vec<Box<dyn Fn(std::sync::Arc<dyn TEvent>, AtomicContextManager) -> Future<R, E> + Send + Sync>>;
pub enum EventHandlers<R, E> {
	Sync(Handlers<R, E>),
	Async(Handlers<R, E>),
}
impl<R, E> EventHandlers<R, E> {
	pub fn extend(&mut self, handlers: Handlers<R, E>) {
		match self {
			Self::Sync(h) => h.extend(handlers),
			Self::Async(h) => h.extend(handlers),
		}
	}
}
pub type TEventHandler<R, E> = HashMap<String, EventHandlers<R, E>>;

/// Task Local Context Manager
/// This is called for every time `handle` method is invoked.
pub struct ContextManager {
	pub event_queue: VecDeque<Arc<dyn TEvent>>,
}

impl ContextManager {
	/// Creation of context manager returns context manager AND event receiver
	pub fn new() -> AtomicContextManager {
		Arc::new(RwLock::new(Self { event_queue: VecDeque::new() }))
	}
}

impl Deref for ContextManager {
	type Target = VecDeque<Arc<dyn TEvent>>;
	fn deref(&self) -> &Self::Target {
		&self.event_queue
	}
}
impl DerefMut for ContextManager {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.event_queue
	}
}

#[async_trait]
pub trait TEventBus<R, E>
where
	R: ApplicationResponse,
	E: ApplicationError + std::convert::From<crate::responses::BaseError> + std::convert::From<E>,
	crate::responses::BaseError: std::convert::From<E>,
{
	fn event_handler(&self) -> &'static TEventHandler<R, E>;
	async fn handle_event(&self, msg: Arc<dyn TEvent>) -> Result<(), E> {
		let context_manager = ContextManager::new();
		self._handle_event(msg, context_manager.clone()).await
	}
	async fn _handle_event(&self, msg: Arc<dyn TEvent>, context_manager: AtomicContextManager) -> Result<(), E> {
		// ! msg.topic() returns the name of event. It is crucial that it corresponds to the key registered on Event Handler.

		let handlers = self.event_handler().get(&msg.metadata().topic).ok_or_else(|| {
			eprintln!("Unprocessable Event Given! {:?}", msg);
			BaseError::NotFound
		})?;

		match handlers {
			EventHandlers::Sync(h) => {
				for handler in h.iter() {
					if let Err(err) = handler(msg.clone(), context_manager.clone()).await {
						// ! Safety:: BaseError Must Be Enforced To Be Accepted As Variant On ServiceError
						match err.into() {
							BaseError::StopSentinel => {
								eprintln!("Stop Sentinel Arrived!");

								break;
							}
							BaseError::StopSentinelWithEvent(event) => {
								eprintln!("Stop Sentinel With Event Arrived!");
								context_manager.write().await.push_back(event);
								break;
							}
							err => {
								eprintln!("Error Occurred While Handling Event! Error:{:?}", err);
							}
						}
					}
				}
			}
			EventHandlers::Async(h) => {
				let mut futures = Vec::new();
				for handler in h.iter() {
					futures.push(handler(msg.clone(), context_manager.clone()));
				}
				let _ = futures::future::try_join_all(futures).await;
			}
		}

		// Resursive case
		let incoming_event = context_manager.write().await.event_queue.pop_front();
		if let Some(event) = incoming_event {
			if let Err(err) = self._handle_event(event, context_manager.clone()).await {
				// ! Safety:: BaseError Must Be Enforced To Be Accepted As Variant On ServiceError
				eprintln!("{:?}", err);
			}
		}
		Ok(())
	}
}

#[async_trait]
pub trait TMessageBus<R, E, C>: TEventBus<R, E>
where
	responses::BaseError: std::convert::From<E>,
	R: ApplicationResponse,
	E: ApplicationError + std::convert::From<crate::responses::BaseError>,
	C: TCommand,
{
	fn command_handler(&self, context_manager: AtomicContextManager) -> impl TCommandService<R, E, C>;

	async fn handle(&self, message: C) -> Result<R, E> {
		let context_manager = ContextManager::new();
		let res = self.command_handler(context_manager.clone()).execute(message).await?;
		// Trigger event
		if !context_manager.read().await.event_queue.is_empty() {
			let event = context_manager.write().await.event_queue.pop_front();
			let _ = self._handle_event(event.unwrap(), context_manager.clone()).await;
		}
		Ok(res)
	}
}

/// This macro is used to create event handler for each event.
/// ## Example
/// ```rust,no_run
/// init_event_handler!(
/// YourServiceResponse,
/// YourServiceError,
/// |ctx| YourEventHandler(ApplicationRepository::new(ctx)),
/// #[asynchronous]
/// YourEvent:[handler1, handler2],
/// #[synchronous]
/// YourEvent2:[handler3, handler4],
/// );
/// ```
#[macro_export]
macro_rules! init_event_handler {
    (
		$R:ty,
		$E:ty,
		$context_handler :expr,
			$(
				$(#[$asynchrony:ident])?
				$event:ty:[$($handler:ident $(=>($($injectable:ident $(( $($arg:ident),* ))? ),*))?),* $(,)? ]
			),*
			$(,)?

    ) =>{
		pub fn event_handler() -> &'static ::ruva::prelude::TEventHandler<$R, $E>  {
			static EVENT_HANDLER: ::std::sync::OnceLock<::ruva::prelude::TEventHandler<$R, $E>> = ::std::sync::OnceLock::new();
			EVENT_HANDLER.get_or_init(||{
				let mut _map : ::ruva::prelude::TEventHandler<$R, $E> = ::ruva::prelude::HandlerMapper::new();
				$(

				let mut handlers = if stringify!($asc) == "asynchronous" {
					::ruva::prelude::EventHandlers::Async(vec![])
				} else {
					::ruva::prelude::EventHandlers::Sync(vec![])
				};
				handlers.extend(vec![
					$(
						Box::new(
							|e: ::std::sync::Arc<dyn TEvent>, context_manager: ::ruva::prelude::AtomicContextManager| -> ::std::pin::Pin<Box<dyn futures::Future<Output = Result<$R, $E>> + Send>>{
								let event_handler = $context_handler(context_manager);
								Box::pin(event_handler.$handler(
									// * Convert event so event handler accepts not Arc<dyn TEvent> but `event_happend` type of message.
									// Safety:: client should access this vector of handlers by providing the corresponding event name
									// So, when it is followed, it logically doesn't make sense to cause an error.
									e.downcast_ref::<$event>().expect("Not Convertible!").clone(),
								))
							}
							),
					)*
				]);
                _map.insert(
                    stringify!($event).into(),
					handlers
                );
            )*
            _map
        })
    }
}

}
