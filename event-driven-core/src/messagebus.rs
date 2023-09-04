use crate::prelude::{Command, Message};
use crate::responses::{ApplicationError, ApplicationResponse, BaseError};
use tokio::sync::{
	mpsc::{channel, error::TryRecvError, Receiver, Sender},
	RwLock,
};

use std::{
	any::{Any, TypeId},
	collections::HashMap,
	pin::Pin,
	sync::Arc,
};

pub type Future<T, E> = Pin<Box<dyn futures::Future<Output = Result<T, E>> + Send>>;
pub type TEventHandler<T, R, E> = HashMap<String, Vec<Box<dyn Fn(Box<dyn Message>, T) -> Future<R, E> + Send + Sync>>>;
pub type TCommandHandler<T, R, E> = HashMap<TypeId, Box<dyn Fn(Box<dyn Any + Send + Sync>, T) -> Future<R, E> + Send + Sync>>;

pub type AtomicContextManager = Arc<RwLock<ContextManager>>;

/// Task Local Context Manager
/// This is called for every time Messagebus.handle is invoked within which it manages events raised in service.
/// It spawns out Executor that manages transaction.
pub struct ContextManager {
	pub sender: Sender<Box<dyn Message>>,
}

impl ContextManager {
	/// Creation of context manager returns context manager AND event receiver
	pub fn new() -> (Arc<RwLock<Self>>, Receiver<Box<dyn Message>>) {
		let (sender, receiver) = channel(20);
		(Arc::new(RwLock::new(Self { sender })), receiver)
	}
}
pub struct MessageBus<
	R: ApplicationResponse,
	E: ApplicationError + std::convert::Into<crate::responses::BaseError> + std::convert::From<crate::responses::BaseError>,
> {
	command_handler: &'static TCommandHandler<AtomicContextManager, R, E>,
	event_handler: &'static TEventHandler<AtomicContextManager, R, E>,
}

impl<
		R: ApplicationResponse,
		E: ApplicationError + std::convert::Into<crate::responses::BaseError> + std::convert::From<crate::responses::BaseError>,
	> MessageBus<R, E>
{
	pub fn new(
		command_handler: &'static TCommandHandler<AtomicContextManager, R, E>,
		event_handler: &'static TEventHandler<AtomicContextManager, R, E>,
	) -> Arc<Self> {
		Self {
			command_handler,
			event_handler,
		}
		.into()
	}

	pub async fn handle<C>(&self, message: C) -> Result<R, E>
	where
		C: Command,
	{
		let (context_manager, mut event_receiver) = ContextManager::new();

		let res = self.command_handler.get(&message.type_id()).ok_or_else(|| {
			eprintln!("Unprocessable Command Given!");
			BaseError::CommandNotFound
		})?(Box::new(message), context_manager.clone())
		.await?;

		'event_handling_loop: loop {
			// * use of try_recv is to stop blocking it when all events are drained.

			match event_receiver.try_recv() {
				// * Logging!
				Ok(msg) => {
					tracing::info!("BreakPoint on OK");

					if let Err(err) = self.handle_event(msg, context_manager.clone()).await {
						// ! Safety:: BaseError Must Be Enforced To Be Accepted As Variant On ServiceError
						if let "EventNotFound" = err.to_string().as_str() {
							continue;
						}
					}
				}
				Err(TryRecvError::Empty) => {
					tracing::info!("BreakPoint on Empty");

					if Arc::strong_count(&context_manager) == 1 {
						break 'event_handling_loop;
					} else {
						continue;
					}
				}
				Err(TryRecvError::Disconnected) => {
					tracing::error!("BreakPoint on Disconnected");
					break 'event_handling_loop;
				}
			};
		}
		drop(context_manager);
		Ok(res)
	}

	async fn handle_event(&self, msg: Box<dyn Message>, context_manager: AtomicContextManager) -> Result<(), E> {
		// ! msg.topic() returns the name of event. It is crucial that it corresponds to the key registered on Event Handler.

		let handlers = self.event_handler.get(&msg.metadata().topic).ok_or_else(|| {
			eprintln!("Unprocessable Event Given! {:?}", msg);
			BaseError::EventNotFound
		})?;

		for handler in handlers.iter() {
			println!("Handle Event : {:?}", msg);
			match handler(msg.message_clone(), context_manager.clone()).await {
				Ok(_val) => {
					println!("Event Handling Succeeded!");
				}

				// ! Safety:: BaseError Must Be Enforced To Be Accepted As Variant On ServiceError
				Err(err) => match err.into() {
					BaseError::StopSentinel => {
						eprintln!("Stop Sentinel Arrived!");

						break;
					}
					BaseError::StopSentinelWithEvent(event) => {
						eprintln!("Stop Sentinel With Event Arrived!");
						context_manager.write().await.sender.send(event).await.expect("Event Collecting failed!");
						break;
					}
					err => {
						eprintln!("Error Occurred While Handling Event! Error:{}", err);
					}
				},
			};
		}
		drop(context_manager);
		Ok(())
	}
}

/// init_command_handler creating macro
#[macro_export]
macro_rules! init_command_handler {
    (
        {$($command:ty:$handler:expr $(=>($($injectable:ident),*))? ),* $(,)?}
    )
        => {
        pub async fn init_command_handler() -> HashMap::<TypeId,Box<dyn Fn(Box<dyn Any + Send + Sync>, AtomicContextManager) -> Future<ServiceResponse,ServiceError> + Send + Sync>>{
            let _dependency= dependency();

            let mut _map: HashMap::<_,Box<dyn Fn(_, _ ) -> Future<_,_> + Send + Sync>> = HashMap::new();
            $(
                _map.insert(
                    // ! Only one command per one handler is acceptable, so the later insertion override preceding one.
                    TypeId::of::<$command>(),
                    Box::new(
                        |c:Box<dyn Any+Send+Sync>, context_manager: AtomicContextManager|->Future<ServiceResponse,ServiceError>{
                            // * Convert event so event handler accepts not Box<dyn Message> but `event_happend` type of message.
                            // ! Logically, as it's from TypId of command, it doesn't make to cause an error.
                            Box::pin($handler(
                                *c.downcast::<$command>().unwrap(),
                                context_manager,
                            $(
                                // * Injectable functions are added here.
                                $(dependency.$injectable(),)*
                            )?
                          ))
                        },
                    )
                );
            )*
            _map
        }
    };
}

/// init_event_handler creating macro
#[macro_export]
macro_rules! init_event_handler {
    (
        {$($event:ty: [$($handler:expr $(=>($($injectable:ident),*))? ),* $(,)? ]),* $(,)?}
    ) =>{
        pub async fn init_event_handler() -> HashMap<String, Vec<Box<dyn Fn(Box<dyn Message>, AtomicContextManager) -> Future<ServiceResponse,ServiceError> + Send + Sync>>>{
            let dependency= dependency();
            let mut _map : HashMap<String, Vec<Box<dyn Fn(_, _) -> Future<_,_> + Send + Sync>>> = HashMap::new();
            $(
                _map.insert(
                    stringify!($event).into(),
                    vec![
                        $(
                            Box::new(
                                |e:Box<dyn Message>, context_manager:AtomicContextManager| -> Future<ServiceResponse,ServiceError>{
                                    Box::pin($handler(
                                        // * Convert event so event handler accepts not Box<dyn Message> but `event_happend` type of message.
                                        // Safety:: client should access this vector of handlers by providing the corresponding event name
                                        // So, when it is followed, it logically doesn't make sense to cause an error.
                                        *e.downcast::<$event>().expect("Not Convertible!"), context_manager,
                                    $(
                                        // * Injectable functions are added here.
                                        $(dependency.$injectable(),)*
                                    )?
                                    ))
                                }
                                ),
                        )*
                    ]
                );
            )*
            _map
        }
    };
}
