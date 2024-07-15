use ruva::prelude::*;

#[test]
fn test_declare_command() {
	// Even without the `Sync`, `Send` and `Debug` contraint on generic type T, it's still valid.
	#[allow(unused)]
	#[derive(TCommand, Debug)]
	struct SomeCommand<T> {
		id: i32,
		name: String,
		foo: i32,
		t_field: T,
	}

	let command = SomeCommand {
		id: 1,
		name: "migo".into(),
		foo: 2,
		t_field: 1,
	};
	assert_eq!(format!("{:?}", command), "SomeCommand { id: 1, name: \"migo\", foo: 2, t_field: 1 }".to_string());
}
