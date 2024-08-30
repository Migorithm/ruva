[ruva-core]: https://docs.rs/ruva-core
[ruva-macro]: https://docs.rs/ruva-macro
[into_command]: https://docs.rs/ruva-macro/latest/ruva_macro/attr.into_command.html
[TEvent]: https://docs.rs/ruva-core/latest/ruva_core/message/trait.TEvent.html
[MessageBus]: https://docs.rs/ruva-core/latest/ruva_core/bus_components/messagebus/index.html
[ContextManager]: https://docs.rs/ruva-core/latest/ruva_core/bus_components/contexts/struct.ContextManager.html
[TCommandService]: https://docs.rs/ruva-core/latest/ruva_core/handler/trait.TCommandService.html


A event-driven framework for writing reliable and scalable system.

At a high level, it provides a few major components:

* Tools for [core components with traits][ruva-core],
* [Macros][ruva-macro] for processing events and commands

# A Tour of Ruva

Ruva consists of a number of modules that provide a range of functionality
essential for implementing messagebus-like applications in Rust. In this
section, we will take a brief tour, summarizing the major APIs and
their uses.

## TCommand & Event
You can register any general struct with [into_command] Derive Macro as follows:
```rust
#[ruva::into_command]
pub struct MakeOrder {
    pub user_id: i64,
    pub items: Vec<String>,
}
```
As you attach [into_command] derive macro, [MessageBus] is now able to understand how and where it should
dispatch the command to.

Likewise, you can do the same thing for Event:
```rust
#[derive(Serialize, Deserialize, Clone, TEvent)]
#[internally_notifiable]
pub struct OrderFailed {
    pub user_id: i64,
}

#[derive(Serialize, Deserialize, Clone, TEvent)]
#[externally_notifiable(OrderAggregate)]
pub struct OrderSucceeded{
    #[identifier]
    pub id: i64,
    pub user_id: i64,
    pub items: Vec<String>
}
```
Notice that `internally_notifiable` event doesn't require aggregate specification while `externally_notifiable` event does along with its id with `identifier` attribute.

* `internally_notifiable` is marker to let the system know that the event should be handled within the application
* `externally_notifiable` event is stored as `OutBox`.

## Initializing TCommandService
For messagebus to recognize service handler, [TCommandService] must be implemented, the response of which is sent directly to
clients.
```rust 
pub struct MessageBus {
event_handler: &'static TEventHandler<ApplicationResponse, ApplicationError>,
}
impl ruva::TMessageBus<ApplicationResponse,ApplicationError, Command> for MessageBus{
fn command_handler(
    &self,
    context_manager: ruva::AtomicContextManager,
    cmd: Command,
) -> impl ruva::TCommandService<ApplicationResponse, ApplicationError> {
    HighestLevelOfAspectThatImplementTCommandService::new(
        MidLevelAspectThatImplementTCommandService::new(
            TargetServiceThatImplementTCommandService::new(
                cmd, other_dependency
            )
        )
    )
}
}
```

For your convenience, Ruva provides declarative macros that handles transaction unit of work as you can use it as follows:

```rust
ruva::register_uow_services!(
	ServiceResponse,
	ServiceError,

	//Command => handler mapping
	CreateUserAccount => create_user_account,
	UpdatePassword => update_password,
    MakeOrder => make_order,
    DeliverProduct => deliver_product
)

```


## Registering Event

`Event` is a side effect of command handling or yet another event processing.
You can register as many handlers as possible as long as they all consume same type of Event as follows:

### Example

```rust
use ruva::ruva_core::init_event_handler;

init_event_handler!(
{
    Response,
    Error,
    |ctx| YourServiceEventHandler::new(ctx),

    OrderFaild: [
           NotificationHandler::send_mail,
           ],
           
    #[async]
    OrderSucceeded: [
           DeliveryHandler::checkout_delivery_items,
           InventoryHandler::change_inventory_count
    ]
}
);
```
In the `MakeOrder` TCommand Handling, we have either `OrderFailed` or `OrderSucceeded` event with their own processing handlers.
Events are raised in the handlers that are thrown to [MessageBus] by [ContextManager].
[MessageBus] then loops through the handlers UNLESS `StopSentinel` is received.

## Handler API Example(Doc required)



## MessageBus
At the core is event driven library is [MessageBus], which gets command and gets raised event from
`UnitOfWork` and dispatch the event to the right handlers.
As this is done only in framework side, the only way you can 'feel' the presence of messagebus is
when you invoke it. Everything else is done magically.

### Example
```rust

#[ruva::into_command]
pub struct MakeOrder { // Test TCommand
    pub user_id: i64,
    pub items: Vec<String>
}

async fn test_func(){
    let bus = MessageBus::new(command_handler(), event_handler())
    let command = MakeOrder{user_id:1, items:vec!["shirts","jeans"]}
    match bus.execute_and_wait(command,Box::new(connection_pool())).await{
        Err(err)=> { // test for error case }
        Ok(val)=> { // test for happy case }
    }
    }
    }   
}
```

#### Error from MessageBus
When command has not yet been regitered, it returns an error - `BaseError::NotFound`
Be mindful that bus does NOT return the result of event processing as in distributed event processing.


