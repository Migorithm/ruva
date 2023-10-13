use syn::{DataEnum, DeriveInput, Field, Ident, Type, Variant};

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

pub(crate) fn find_attr_and_locate_its_type_from_field(field: &mut Field, attribute_name: &str) -> Vec<Type> {
	let mut identifier_types = vec![];
	for attr in field.attrs.iter_mut() {
		if attr.meta.path().segments.iter().any(|f| f.ident == *attribute_name) {
			identifier_types.push(field.ty.clone());
		}
	}
	identifier_types
}
