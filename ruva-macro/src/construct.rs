use proc_macro2::TokenStream;

/// extract all the fields from the struct and type, then create a new instance of the struct with the associated function named construct
/// construct function will take all the fields as arguments and return the struct instance
/// with #[except] attribute, the field will be excluded from the construct function.
/// With the use of #[except] attribute, the struct must derive Default to be able to construct the struct
/// ```rust
/// #[derive(Default,TConstruct)]
/// struct TestStruct {
/// value: i32,
/// #[except]
/// name: String,
/// }
///
/// let test = TestStruct::construct(1);
/// assert_eq!(test.value, 1);
/// assert_eq!(test.name, String::default());
/// ```
///
pub fn expand_derive_construct(input: &mut syn::DeriveInput) -> syn::Result<TokenStream> {
	let name = &input.ident;
	let generics = &input.generics;
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let fields = match &input.data {
		syn::Data::Struct(data) => match &data.fields {
			syn::Fields::Named(fields) => fields.named.iter().map(|field| {
				let ident = field.ident.as_ref().unwrap();
				let ty = &field.ty;
				let attrs = &field.attrs;
				(ident, ty, attrs)
			}),
			_ => return Err(syn::Error::new_spanned(input, "Only Structs with named fields are supported")),
		},
		_ => return Err(syn::Error::new_spanned(input, "Only Structs are supported")),
	};

	let field_len = fields.len();

	let mut input_arguments = Vec::with_capacity(field_len);
	let mut struct_fields = Vec::with_capacity(field_len);

	fields.for_each(|(ident, ty, attrs)| {
		let ident = ident.clone();
		let ty = ty.clone();
		let attrs = attrs.clone();
		let except = attrs.iter().any(|attr| {
			if let syn::Meta::Path(path) = &attr.meta {
				if path.is_ident("except") {
					return true;
				}
			}
			false
		});

		if !except {
			input_arguments.push(quote!(#ident: #ty));
			struct_fields.push(quote!(#ident))
		}
	});

	let construct = if input_arguments.len() == field_len {
		quote! {
			impl #impl_generics #name #ty_generics #where_clause {
				// imploement the construct function for the struct
				// the function will take all the fields as arguments and return the struct instance
				pub fn construct(#(#input_arguments),*) -> Self {
					Self {
						#(#struct_fields),*
					}
				}
			}
		}
	} else {
		quote! {
			impl #impl_generics #name #ty_generics #where_clause {
				pub fn construct(#(#input_arguments),*) -> Self {
					Self {
						#(#struct_fields),*
						,..Default::default()
					}
				}
			}
		}
	};

	Ok(construct)
}
