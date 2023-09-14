use proc_macro::TokenStream;
use syn::{Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, Ident, Meta, Path};

pub(crate) fn render_message_token(ast: &DeriveInput, propagatability: Vec<&'static str>, identifier: String) -> TokenStream {
	let name = &ast.ident;

	let identifier: proc_macro2::TokenStream = identifier.parse().unwrap();
	let joined: proc_macro2::TokenStream = propagatability.join(" ").parse().unwrap();

	quote! {
		impl Message for #name {

			#identifier

			fn message_clone(&self) -> Box<dyn Message> {
				Box::new(self.clone())
			}
			fn state(&self) -> String {
				serde_json::to_string(&self).expect("Failed to serialize")
			}
			fn to_message(self)-> Box<dyn Message+'static>{
				Box::new(self)
			}

			#joined
		}
		impl MailSendable for #name {
			fn template_name(&self) -> String {
				// * subject to change
				stringify!($name).into()
			}
		}
	}
	.into()
}

pub(crate) fn render_event_visibility(ast: &DeriveInput) -> Vec<&'static str> {
	let propagatability = ast
		.attrs
		.iter()
		.flat_map(|attr| {
			if let Meta::Path(Path { segments, .. }) = &attr.meta {
				segments
					.iter()
					.filter_map(|s| {
						if s.ident.to_string().as_str() == "internally_notifiable" {
							Some("fn internally_notifiable(&self)->bool{true}")
						} else if s.ident.to_string().as_str() == "externally_notifiable" {
							Some("fn externally_notifiable(&self)->bool{true}")
						} else {
							None
						}
					})
					.collect::<Vec<_>>()
			} else {
				panic!("Error!")
			}
		})
		.collect::<Vec<_>>();
	if propagatability.is_empty() {
		panic!("Either `internally_notifiable` or `externally_notifiable` must be given!")
	}
	propagatability
}

pub(crate) fn find_identifier(ast: &DeriveInput) -> String {
	let name = &ast.ident;
	match &ast.data {
		Data::Struct(DataStruct {
			fields: Fields::Named(FieldsNamed { named, .. }),
			..
		}) => {
			let identifier = named.iter().filter(|f| get_attributes(f).into_iter().any(|ident| ident == *"identifier")).collect::<Vec<_>>();
			if identifier.len() != 1 {
				panic!("One identifier Must Be Given To Message!")
			}

			let ident = identifier.first().unwrap().ident.clone().unwrap().clone();

			format!(
				"
				fn metadata(&self) -> MessageMetadata {{
					MessageMetadata{{
					aggregate_id: self.{}.to_string(),
					topic: stringify!({}).into()

				}}
			}}",
				ident, name
			)
		}
		_ => panic!("Only Struct Allowed!"),
	}
}

fn get_attributes(field: &Field) -> Vec<Ident> {
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
