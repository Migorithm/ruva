use std::borrow::Borrow;

use syn::{parse::Parser, parse_quote, punctuated::Punctuated, DataEnum, DeriveInput, Field, FieldsNamed, Ident, Meta, Path, Stmt, Type, Variant};

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

// tell if a field is annotated with specific attribute name and get its Types
#[allow(unused)]
pub(crate) fn find_attr_and_locate_its_type_from_field(field: &mut Field, attribute_name: &str) -> Vec<Type> {
	let mut types_found = vec![];
	for attr in field.attrs.iter_mut() {
		if attr.meta.path().segments.iter().any(|f| f.ident == *attribute_name) {
			types_found.push(field.ty.clone());
		}
	}
	types_found
}

pub(crate) fn check_if_field_has_attribute_and_return_field_name(field: &Field, attribute_name: &str) -> Option<String> {
	let mut field_name = None;
	for attr in field.attrs.iter() {
		if attr.path().is_ident(attribute_name) {
			field_name = Some(field.ident.as_ref().unwrap().to_string());
		}
	}
	field_name
}

pub(crate) fn extracts_field_names_from_derive_input(ast: &DeriveInput) -> Vec<String> {
	match &ast.data {
		syn::Data::Struct(syn::DataStruct {
			fields: syn::Fields::Named(fields), ..
		}) => fields.named.iter().map(|f| f.ident.clone().unwrap().to_string()).collect(),
		_ => panic!("Only Struct Is supported"),
	}
}

pub(crate) fn remove_fields_from_fields_based_on_field_name(given_fields: &mut syn::FieldsNamed, fields_to_remove: impl Borrow<Vec<String>>) {
	let fields_borrow = fields_to_remove.borrow();

	let new_fields = given_fields
		.named
		.iter()
		.filter(|f| if let Some(ident) = &f.ident { !fields_borrow.contains(&ident.to_string()) } else { true })
		.map(|f| syn::Field::parse_named.parse2(quote!( #f )).unwrap())
		.collect::<Punctuated<Field, _>>();

	let res = FieldsNamed {
		brace_token: syn::token::Brace::default(),
		named: new_fields,
	};

	*given_fields = res;
}

pub(crate) fn skip_over_attributes(field: &mut Field, attribute_name: &str) {
	field.attrs.retain(|attr| !attr.path().is_ident(attribute_name));
}

// get attributes from field
pub(crate) fn get_attributes(field: &Field) -> Vec<Ident> {
	let Field { attrs, .. } = field;
	{
		let mut attributes = attrs
			.iter()
			.flat_map(|attr| match &attr.meta {
				Meta::Path(Path { segments, .. }) => segments.iter().map(|segment| segment.ident.clone()).collect::<Vec<Ident>>(),
				_ => panic!("Only Path"),
			})
			.collect::<Vec<_>>();
		attributes.sort();
		attributes
	}
}
pub(crate) fn get_type_name(ty: &Type) -> String {
	let syn::Type::Path(syn::TypePath { path, .. }) = ty else { panic!("Wrong type") };
	path.segments.first().unwrap().ident.to_string()
}
pub(crate) fn get_trait_checking_stmts(trait_path: &str) -> Vec<Stmt> {
	let path = syn::parse::<Path>(trait_path.parse().expect("Unqualified path")).expect("Parsing path for trait failed!");

	vec![
		// Blacket implementation for Type T
		parse_quote!(
			trait __IsTraitNotImplemented {
				const IS_TRAIT: bool = false;

				fn get_data<T>(_: impl std::any::Any) -> &'static mut T {
					unreachable!()
				}
			}
		),
		parse_quote!(
			impl<T> __IsTraitNotImplemented for T {}
		),
		// Blacket implementation for Type T that implements TAggregate
		parse_quote!(
			struct IsTrait<T>(::core::marker::PhantomData<T>);
		),
		parse_quote!(
			#[allow(unused)]
			impl<T: #path> IsTrait<T> {
				const IS_TRAIT: bool = true;

				fn get_data(data: &mut T) -> &mut T {
					data
				}
			}
		),
	]
}
