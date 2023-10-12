use syn::{DataEnum, DeriveInput, Ident, Variant};

pub(crate) fn locate_crate_on_derive_macro(ast: &DeriveInput) -> Ident {
	let crates = ast.attrs.iter().find(|x| x.path().is_ident("crates"));
	let crates = if let Some(crates) = crates {
		crates.parse_args::<syn::ExprPath>().unwrap().path.get_ident().expect("#[crates(...)] expects path.").to_string()
	} else {
		"ruva".to_owned()
	};
	syn::Ident::new(&crates, proc_macro2::Span::call_site())
}

pub(crate) fn find_enum_variant<'a>(data_enum: &'a DataEnum) -> impl Fn(&'a str) -> Option<&'a Variant> {
	|name: &str| data_enum.variants.iter().find(|x| x.attrs.iter().any(|x| x.path().is_ident(name)))
}
