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

	let parsed_data = parse_macro_data(&data);

	"".parse().expect("잘못된 문법입니다.")
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
	input.as_ref().replace(" ", "").replace("\n", "")
}
