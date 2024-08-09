use ruva::*;
use ruva_macro::into_command;

#[test]
fn test_into_command_with() {
	#[into_command(Serialize)]
	struct SomeCommand {
		#[required_input]
		id: i32,
		name: String,
		foo: i32,
	}

	let command = SomeCommandBody { name: "migo".into(), foo: 2 };
	let serilaized = serde_json::to_string(&command).unwrap();
	let deserialized: SomeCommandBody = serde_json::from_str::<SomeCommandBody>(&serilaized).unwrap();
	assert_eq!(format!("{:?}", deserialized), "SomeCommandBody { name: \"migo\", foo: 2 }".to_string());

	let command2 = deserialized.into_command(1);
	assert_eq!(format!("{:?}", command2), "SomeCommand { id: 1, name: \"migo\", foo: 2 }".to_string());
}

#[test]
fn test_into_command_with_generic() {
	// Even without the `Sync`, `Send` and `Debug` contraint on generic type T, it's still valid.

	#[into_command(Serialize)]
	struct SomeCommand<T: Serialize> {
		#[required_input]
		id: i32,
		name: String,
		foo: i32,
		t_field: T,
	}

	let command = SomeCommandBody::<i32> {
		name: "migo".into(),
		foo: 2,
		t_field: 1,
	};
	let serilaized = serde_json::to_string(&command).unwrap();
	let deserialized: SomeCommandBody<i32> = serde_json::from_str::<SomeCommandBody<i32>>(&serilaized).unwrap();
	assert_eq!(format!("{:?}", deserialized), "SomeCommandBody { name: \"migo\", foo: 2, t_field: 1 }".to_string());

	let command2 = deserialized.into_command(1);
	assert_eq!(format!("{:?}", command2), "SomeCommand { id: 1, name: \"migo\", foo: 2, t_field: 1 }".to_string());
}
