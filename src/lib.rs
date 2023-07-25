#[macro_use]
extern crate macro_rules_attribute;
// pub mod database;
pub mod domain;
pub mod messagebus;
pub mod outbox;
pub mod repository;
pub mod responses;

pub mod prelude {
    pub use crate::count;
    pub use crate::domain::{Aggregate, Buildable, Builder, Message};
    pub use crate::Aggregate as AggregateMacro;
    pub use serde::ser::SerializeStruct;
}
pub use prelude::*;
