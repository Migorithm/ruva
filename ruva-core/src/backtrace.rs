#[inline]
pub fn get_caller_data() -> Option<String> {
	let stack_trace = backtrace::Backtrace::new();
	let caller = stack_trace
		.frames()
		.iter()
		.filter(|x| x.symbols().first().and_then(|x| x.name()).and_then(|x| x.as_str()).is_some())
		.filter(|x| {
			static BLACKLIST: [&str; 7] = ["backtrace::", "ruva_core::", "tokio::", "core::", "std::", "test::", "futures::"];
			let name = x.symbols().first().and_then(|x| x.name()).and_then(|x| x.as_str()).unwrap();
			if BLACKLIST.iter().any(|y| name.starts_with(y)) {
				return false;
			}
			true
		})
		.map(|x| x.symbols().first().unwrap())
		.next()
		.cloned()?;

	let module_path = caller.name().expect("caller에서 name을 기준으로 비교했기 때문에 현재는 값이 반드시 존재함");
	let filename = caller.filename();
	let line = caller.lineno();

	let mut result = String::new();
	result.push_str(module_path.as_str().unwrap());
	if let Some(file_path) = filename {
		if let Some(file_path) = file_path.to_str() {
			result.push(' ');
			result.push_str(file_path);
		}
	}

	if let Some(line_no) = line {
		result.push(':');
		result.push_str(line_no.to_string().as_str());
	}
	Some(result)
}

#[macro_export]
macro_rules! backtrace_error {
    ($($arg:tt)*) => {
        #[cfg(feature = "backtrace")]
        {
            let caller = $crate::backtrace::get_caller_data();
            if caller.is_some() {
                let caller = caller.unwrap();
                tracing::error!(?caller, $($arg)*);
            } else {
                tracing::error!($($arg)*);
            }
        }
        #[cfg(not(feature = "backtrace"))]
        {
            tracing::error!($($arg)*);
        }
    };
}
