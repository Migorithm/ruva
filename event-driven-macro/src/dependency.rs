use proc_macro::TokenStream;
use syn::{ItemFn, ReturnType, Signature};

pub fn register_dependency(ast: ItemFn) -> TokenStream {
	const OUTPUT_TYPE_NOT_VALID: &str = "#[dependency] fn must have valid output type";

	let ItemFn {
		sig: Signature {
			ident,
			output: ReturnType::Type(_, var),
			inputs,
			generics,
			asyncness,
			..
		},
		block,
		..
	} = ast.clone()
	else {
		panic!("{}", OUTPUT_TYPE_NOT_VALID)
	};

	// check return type is not void '()'
	if let syn::Type::Tuple(tuple) = var.as_ref() {
		if tuple.elems.is_empty() {
			panic!("{}", OUTPUT_TYPE_NOT_VALID)
		}
	}

	quote!(
	impl Dependency{
		pub #asyncness fn #ident #generics(&self,#inputs)-> #var{
			#block
		}
	}
	#[allow(dead_code)]
	#ast
	)
	.into()
}
