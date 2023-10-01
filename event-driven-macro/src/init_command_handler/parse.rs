use super::MacroDataSingle;
use nom::IResult;

pub(super) fn parse_macro_data(data: &str) -> Vec<MacroDataSingle> {
	let mut output: Vec<MacroDataSingle> = Default::default();

	let mut remain = data;
	loop {
		let result = get_one_stream(remain);
		if result.is_err() {
			// 다음 ,가 없다는 뜻으로, 끝났다는 것
			break;
		}
		let result = result.unwrap();
		let line = result.1;

		output.push(parse(line));

		remain = &result.0[1..];
	}
	output.push(parse(remain));

	output
}

/// (",etc", line)
fn get_one_stream(input: &str) -> IResult<&str, &str> {
	nom::bytes::streaming::take_while(|c| c != ',')(input)
}

fn parse(line: &str) -> MacroDataSingle {
	let (etc, command) = get_command(line);
	let (etc, handler) = get_handler(etc);
	if etc.is_empty() {
		MacroDataSingle { command, handler, injectable: vec![] }
	} else {
		MacroDataSingle {
			command,
			handler,
			injectable: get_injectables(etc),
		}
	}
}

/// (etc, command) (: 없음)
fn get_command(input: &str) -> (&str, &str) {
	let result: IResult<_, _> = nom::bytes::streaming::take_while(|c| c != ':')(input);
	let result = result.expect("init_command_handler 내부 라인 중 :가 오지 않았습니다.");
	(&result.0[1..], result.1)
}

/// (etc, handler) (=> 없음) (=>가 원래 없었으면, 즉 injectable이 없었으면 etc는 공백임)
fn get_handler(input: &str) -> (&str, &str) {
	let result: IResult<_, _> = nom::bytes::streaming::take_while(|c| c != '=')(input);
	if let Ok(result) = result {
		if result.0.starts_with("=>") {
			(&result.0[2..], result.1)
		} else {
			panic!("init_command_handler 내부 올바르지 않은 =기호입니다.")
		}
	} else {
		("", input)
	}
}

fn get_injectables(input: &str) -> Vec<&str> {
	let mut output = vec![];
	let mut remain = input;
	loop {
		if remain.is_empty() {
			break;
		}
		let result = get_injectable(remain);
		output.push(result.1);
		remain = result.0;
	}
	output
}

/// (etc, injectable) (, 없음) (마지막이면 etc 공백)
fn get_injectable(input: &str) -> (&str, &str) {
	let result: IResult<_, _> = nom::bytes::streaming::take_while(|c| c != ',')(input);
	if let Ok(result) = result {
		if result.0.starts_with(",") {
			(&result.0[1..], result.1)
		} else {
			(result.0, result.1)
		}
	} else {
		("", input)
	}
}
