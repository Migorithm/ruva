use proc_macro::TokenStream;
use syn::{ItemFn, ReturnType, Signature};

pub fn register_dependency(ast: ItemFn) -> TokenStream {
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
		panic!("Fail!")
	};

	quote!(
	impl Dependency{
		pub #asyncness fn #ident #generics(&self,#inputs)-> #var{
			#block

		}
	}
	#ast
	)
	.into()
}
