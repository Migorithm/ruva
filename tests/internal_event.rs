use ruva::*;

/// ### Internally Notifiable Event
/// when annotated with internally notifiable, the event will be handled internally by `MessageBus`.
#[test]
fn test_declare_internal_event() {
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
	#[internally_notifiable]
	pub struct SomeInternalEvent {
		id: i32,
		name: String,
		foo: i32,
	}

	let event = SomeInternalEvent { id: 1, name: "migo".into(), foo: 2 }.to_message();
	let metadata = event.metadata();
	assert_eq!(metadata.aggregate_id, "");
	assert_eq!(metadata.aggregate_name, "");
	assert_eq!(metadata.topic, "SomeInternalEvent");
}
