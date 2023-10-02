mod parse;
use parse::parse_macro_data;
use proc_macro::TokenStream;

struct MacroDataSingle<'data> {
	command: &'data str,
	handler: &'data str,
	injectable: Vec<&'data str>,
}

pub(super) fn init_command_handler(tokens: TokenStream) -> TokenStream {
	let data = tokens.to_string();
	let data = remove_space(data);
	let data = ignore_brace(&data);

	let parsed_data = parse_macro_data(data);

	generate_code(parsed_data).parse().expect("잘못된 문법입니다.")
}

fn ignore_brace(input: &str) -> &str {
	if input.starts_with('{') && input.ends_with('}') {
		&input[1..input.len() - 1]
	} else {
		input
	}
}

#[inline]
fn remove_space(input: impl AsRef<str>) -> String {
	// Multi replacement
	input.as_ref().replace([' ', '\n'], "")
}

fn generate_code(data: Vec<MacroDataSingle>) -> String {
	let mut result = String::new();
	result.push_str(
		"
		pub fn command_handler() -> &'static TCommandHandler<ServiceResponse, ServiceError> {
			extern crate self as current_crate;
			static COMMAND_HANDLER: ::std::sync::OnceLock<TCommandHandler<ServiceResponse, ServiceError>> = OnceLock::new();

			COMMAND_HANDLER.get_or_init(||{
				let dependency= current_crate::dependencies::dependency();
				let mut _map: TCommandHandler<ServiceResponse,ServiceError>= event_driven_library::prelude::HandlerMapper::new();",
	);
	for data in data.into_iter() {
		let command = data.command;
		let handler = data.handler;
		let injectables = data.injectable;
		let injectable = if injectables.is_empty() {
			"".to_string()
		} else {
			injectables.into_iter().map(|injectable| format!("dependency.{}()", injectable)).collect::<Vec<String>>().join(",")
		};
		result.push_str(
			format!(
				"_map.insert(
						// ! Only one command per one handler is acceptable, so the later insertion override preceding one.
						TypeId::of::<{command}>(),

							Box::new(|c:Box<dyn Any+Send+Sync>, context_manager: event_driven_library::prelude::AtomicContextManager|->Future<ServiceResponse,ServiceError> {{
								// * Convert event so event handler accepts not Box<dyn Message> but `event_happend` type of message.
								// ! Logically, as it's from TypId of command, it doesn't make to cause an error.
								Box::pin({handler}(
									*c.downcast::<{command}>().unwrap(),
									context_manager,
									{injectable}
								))
							}},
					));"
			)
			.as_str(),
		);
	}

	result.push_str(
		"
				_map
			})

		}",
	);
	result
}
