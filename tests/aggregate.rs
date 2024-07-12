use ruva::prelude::*;

#[test]
fn test_serialize() {
	#[aggregate]
	#[derive(Debug, Clone, Serialize, Default)]
	pub struct SerializeTest {
		#[adapter_ignore]
		id: i32,
		#[serde(skip_serializing)]
		name: String,
		foo: i32,
	}
	let aggregate = SerializeTest::default();
	let serialized = serde_json::to_string(&aggregate).unwrap();
	assert_eq!(serialized, "{\"id\":0,\"foo\":0}");
}

#[test]
fn test_adapter_accessible() {
	#[aggregate]
	#[derive(Debug, Clone, Serialize, Default)]
	pub struct TestStruct {
		#[adapter_ignore]
		id: i32,
		#[serde(skip_serializing)]
		name: String,
		foo: i32,
	}
	let adapter = TestStructAdapter::default();
	let serialized = serde_json::to_string(&adapter).unwrap();
	assert_eq!(serialized, "{\"foo\":0}");
}

#[test]
fn test_conversion() {
	#[aggregate]
	#[derive(Debug, Clone, Serialize, Default)]
	pub struct ConversionStruct {
		#[adapter_ignore]
		id: i32,
		#[serde(skip_serializing)]
		name: String,
		foo: i32,
	}
	let aggregate = ConversionStruct {
		name: "migo".into(),
		foo: 2,
		id: 1,
		..Default::default()
	};
	assert_eq!(aggregate.id, 1);
	let converted_adapter = ConversionStructAdapter::from(aggregate);

	assert_eq!(converted_adapter.name, "migo");
	assert_eq!(converted_adapter.foo, 2);

	let converted_struct = ConversionStruct::from(converted_adapter);
	assert_eq!(converted_struct.name, "migo");
	assert_eq!(converted_struct.foo, 2);
}

#[test]
fn test_when_there_is_no_apdater_ignore_attr() {
	#[aggregate]
	#[derive(Debug, Clone, Serialize, Default)]
	pub struct TestStruct {
		id: i32,
		name: String,
		some_other_field: i32,
	}

	let non_adapter = TestStruct::default();
	let non_adapter_serialized = serde_json::to_string(&non_adapter).unwrap();
	assert_eq!(non_adapter_serialized, "{\"id\":0,\"name\":\"\",\"some_other_field\":0}");

	let adapter = TestStructAdapter::default();
	let adapter_serialized = serde_json::to_string(&adapter).unwrap();
	assert_eq!(adapter_serialized, "{\"id\":0,\"name\":\"\",\"some_other_field\":0}");
}
