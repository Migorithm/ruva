use ruva::prelude::*;

/// ### Externally Notifiable Event
/// when annotated with externally notifiable, it is stored as outbox.
#[test]
fn test_declare_external_event() {
	#[aggregate]
	#[derive(Debug, Clone, Serialize, Default)]
	pub struct SomeAggregate {
		#[adapter_ignore]
		id: i32,
		#[serde(skip_serializing)]
		name: String,
		foo: i32,
	}

	#[derive(Debug, Clone, Serialize, Default, TEvent)]
	#[externally_notifiable(SomeAggregate)]
	pub struct SomeExternalEvent {
		#[identifier]
		id: i32,
		name: String,
		foo: i32,
	}

	let event = SomeExternalEvent { id: 1, name: "migo".into(), foo: 2 }.to_message();

	let metadata = event.metadata();
	assert_eq!(metadata.aggregate_id, "1");
	assert_eq!(metadata.aggregate_name, "SomeAggregate");
	assert_eq!(metadata.topic, "SomeExternalEvent");
	assert_eq!(event.state(), "{\"id\":1,\"name\":\"migo\",\"foo\":2}");
}
