use super::contexts::*;
use super::handler::{EventHandlers, TCommandService, TEventHandler};
use crate::prelude::{TCommand, TEvent};
use crate::responses::{self, ApplicationError, ApplicationResponse, BaseError};
use async_recursion::async_recursion;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait TEventBus<R, E>
where
	R: ApplicationResponse,
	E: ApplicationError + std::convert::From<crate::responses::BaseError> + std::convert::From<E>,
	crate::responses::BaseError: std::convert::From<E>,
{
	fn event_handler(&self) -> &'static TEventHandler<R, E>;
}

/// This function is used to handle event. It is called recursively until there is no event left in the queue.
#[async_recursion]
async fn handle_event<R, E>(msg: Arc<dyn TEvent>, context_manager: AtomicContextManager, event_handler: &'static TEventHandler<R, E>) -> Result<(), E>
where
	R: ApplicationResponse,
	E: ApplicationError + std::convert::From<crate::responses::BaseError> + std::convert::From<E>,
	crate::responses::BaseError: std::convert::From<E>,
{
	// ! msg.topic() returns the name of event. It is crucial that it corresponds to the key registered on Event Handler.

	let handlers = event_handler.get(&msg.metadata().topic).ok_or_else(|| {
		tracing::error!("Unprocessable Event Given! {:?}", msg);
		BaseError::NotFound
	})?;

	match handlers {
		EventHandlers::Sync(h) => {
			for (i, handler) in h.iter().enumerate() {
				if let Err(err) = handler(msg.clone(), context_manager.clone()).await {
					// ! Safety:: BaseError Must Be Enforced To Be Accepted As Variant On ServiceError
					match err.into() {
						BaseError::StopSentinel => {
							let error_msg = format!("Stop Sentinel Arrived In {i}th Event!");
							crate::backtrace_error!("{}", error_msg);
							break;
						}
						BaseError::StopSentinelWithEvent(event) => {
							let error_msg = format!("Stop Sentinel With Event Arrived In {i}th Event!");
							crate::backtrace_error!("{}", error_msg);
							context_manager.write().await.push_back(event);
							break;
						}
						err => {
							let error_msg = format!("Error Occurred While Handling Event In {i}th Event! Error:{:?}", err);
							crate::backtrace_error!("{}", error_msg);
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
			if let Err(err) = futures::future::try_join_all(futures).await {
				let error_msg = format!("Error Occurred While Handling Event! Error:{:?}", err);
				crate::backtrace_error!("{}", error_msg);
			}
		}
	}

	// Resursive case
	let incoming_event = context_manager.write().await.event_queue.pop_front();
	if let Some(event) = incoming_event {
		if let Err(err) = handle_event(event, context_manager.clone(), event_handler).await {
			// ! Safety:: BaseError Must Be Enforced To Be Accepted As Variant On ServiceError
			tracing::error!("{:?}", err);
		}
	}
	Ok(())
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

	/// This method is used to handle command and return result.
	/// ## Example
	/// ```rust,no_run
	/// let res = service.execute_and_wait(message).await?;
	/// ```
	async fn execute_and_wait(&self, message: C) -> Result<R, E> {
		let context_manager = ContextManager::new();
		let res = self.command_handler(context_manager.clone()).execute(message).await?;

		// Trigger event handler
		if !context_manager.read().await.event_queue.is_empty() {
			let event = context_manager.write().await.event_queue.pop_front();
			let _s = handle_event(event.unwrap(), context_manager.clone(), self.event_handler()).await?;
		}
		Ok(res)
	}

	/// This method is used to handle command and return result proxy which holds the result and join handler.
	/// ## Example
	/// ```rust,no_run
	/// let res = service.execute_and_forget(message).await?;
	/// let res = res.wait_until_event_processing_done().await?;
	/// let res = res.result();
	/// ```
	async fn execute_and_forget(&self, message: C) -> Result<CommandResponseWithEventFutures<R, E>, E> {
		let context_manager = ContextManager::new();
		let res = self.command_handler(context_manager.clone()).execute(message).await?;
		let mut res = CommandResponseWithEventFutures { result: res, join_handler: None };

		// Trigger event handler
		if !context_manager.read().await.event_queue.is_empty() {
			let event = context_manager.write().await.event_queue.pop_front();
			res.join_handler = Some(tokio::spawn(handle_event(event.unwrap(), context_manager.clone(), self.event_handler())));
		}
		Ok(res)
	}
}

pub struct CommandResponseWithEventFutures<T, E> {
	result: T,
	join_handler: Option<tokio::task::JoinHandle<std::result::Result<(), E>>>,
}
impl<T, E> CommandResponseWithEventFutures<T, E>
where
	responses::BaseError: std::convert::From<E>,
	T: ApplicationResponse,
	E: ApplicationError + std::convert::From<crate::responses::BaseError>,
{
	pub async fn wait_until_event_processing_done(mut self) -> Result<Self, E> {
		if let Some(join_handler) = self.join_handler.take() {
			join_handler.await.map_err(|err| {
				tracing::error!("{:?}", err);
				BaseError::ServiceError(Box::new("error occurred while handling event".to_string()))
			})??;
		}
		Ok(self)
	}
	pub fn result(self) -> T {
		self.result
	}
}

/// This macro is used to create event handler for each event.
/// ## Example
/// ```rust,no_run
/// init_event_handler!(
///     YourServiceResponse,
///     YourServiceError,
///     |ctx| YourEventHandler(ApplicationRepository::new(ctx)),
///     #[asynchronous]
///     YourEvent:[handler1, handler2],
///     #[synchronous]
///     YourEvent2:[handler3, handler4],
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
							|e: ::std::sync::Arc<dyn ::ruva::prelude::TEvent>, context_manager: ::ruva::prelude::AtomicContextManager| -> ::std::pin::Pin<Box<dyn futures::Future<Output = Result<$R, $E>> + Send>>{
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
