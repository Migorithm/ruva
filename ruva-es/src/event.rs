use ruva_core::prelude::{
	serde::{de::DeserializeOwned, Serialize},
	serde_json::Value,
};

pub trait TEvent: Serialize + DeserializeOwned + Clone + PartialEq + std::fmt::Debug + Sync + Send {
	/// for event upcasting.
	fn event_type(&self) -> String;
	/// used for event upcasting.
	fn event_version(&self) -> String;
	fn aggregate_type(&self) -> String;
}

pub struct EventEnvolope {
	pub aggregate_type: String,
	pub aggregate_id: String,
	pub sequence: i64,
	pub event_type: String,
	pub event_version: String,
	pub payload: Value,
}
