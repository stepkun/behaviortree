// Copyright © 2025 Stephan Kunz

//! Common derive macro implementation

#[doc(hidden)]
extern crate proc_macro;

#[doc(hidden)]
extern crate alloc;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Field, Ident, Type};

/// Structure for the attributes on struct level
#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(behavior))]
struct BehaviorStructAttributes {
	/// optional attribute `no_create`
	#[deluxe(default = false)]
	no_create: bool,
	/// optional attribute `no_register`
	#[deluxe(default = false)]
	no_register: bool,
	/// optional attribute `no_register_with`
	#[deluxe(default = false)]
	no_register_with: bool,
	/// optional attribute `groot2`
	#[deluxe(default = false)]
	groot2: bool,
}

/// Structure for the attributes on field level
#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(behavior))]
struct BehaviorFieldAttributes {
	/// optional attribute `parameter`
	#[deluxe(default = false)]
	parameter: bool,
}

/// Extracts attributes of fields
fn extract_field_attributes(ast: &mut DeriveInput) -> deluxe::Result<(Vec<Field>, Vec<Ident>, Vec<Type>, usize)> {
	let mut parameters = Vec::new();
	let mut arguments = Vec::new();
	let mut types = Vec::new();
	let mut count = 0;
	if let syn::Data::Struct(s) = &mut ast.data {
		for field in s.fields.iter_mut() {
			count += 1;
			let attrs: BehaviorFieldAttributes = deluxe::extract_attributes(field)?;
			if attrs.parameter {
				let field_name = field.ident.as_ref().unwrap().clone();
				let field_type = field.ty.clone();
				parameters.push(field.clone());
				arguments.push(field_name);
				types.push(field_type);
			}
		}
	}
	Ok((parameters, arguments, types, count))
}

/// Implementation of the derive macro
pub fn derive_behavior_struct(input: TokenStream, kind: super::Kind) -> deluxe::Result<TokenStream> {
	// Construct a representation of the Rust code
	let mut ast: DeriveInput = syn::parse2(input).expect("could not parse input");

	// Check type of input
	match &ast.data {
		syn::Data::Struct(_struct) => {}
		syn::Data::Enum(_enum) => panic!("enums not supported"),
		syn::Data::Union(_union) => panic!("unions not supported"),
	}

	// extract attributes
	let BehaviorStructAttributes {
		no_create,
		no_register,
		no_register_with,
		groot2,
	} = deluxe::extract_attributes(&mut ast)?;

	// dbg!(create, register, register_with, groot2);

	// extract parameter fields
	let (parameter, arguments, types, count) = extract_field_attributes(&mut ast)?;
	assert_eq!(parameter.len(), arguments.len());
	assert_eq!(parameter.len(), types.len());
	let no_params = parameter.len() == 0;

	// structure name
	let ident = &ast.ident;
	let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

	let derived: TokenStream = "#[automatically_derived]"
		.parse()
		.expect("derive(Behavior) - derived");
	let diagnostic: TokenStream = "#[diagnostic::do_not_recommend]"
		.parse()
		.expect("derive(Behavior) - diagnostic");

	let kind_token = match kind {
		crate::Kind::Action => quote! { behaviortree::behavior::BehaviorKind::Action },
		crate::Kind::Condition => quote! { behaviortree::behavior::BehaviorKind::Condition },
		crate::Kind::Control => quote! { behaviortree::behavior::BehaviorKind::Control },
		crate::Kind::Decorator => quote! { behaviortree::behavior::BehaviorKind::Decorator },
	};

	let fn_create_fn = if no_create {
		quote! {}
	} else {
		if arguments.len() == 0 {
			if count == 0 {
				quote! {
					/// Behavior creation function
					#[inline]
					fn create_fn(#(#parameter),*) -> alloc::boxed::Box<behaviortree::behavior::BehaviorCreationFn>  {
						alloc::boxed::Box::new(|| alloc::boxed::Box::new(Self{}))
					}
				}
			} else {
				quote! {
					/// Behavior creation function
					#[inline]
					fn create_fn(#(#parameter),*) -> alloc::boxed::Box<behaviortree::behavior::BehaviorCreationFn>  {
						alloc::boxed::Box::new(|| alloc::boxed::Box::new(Self {..Default::default()}))
					}
				}
			}
		} else {
			if count == arguments.len() {
				quote! {
					/// Behavior creation function
					#[inline]
					fn create_fn(#(#parameter),*) -> alloc::boxed::Box<behaviortree::behavior::BehaviorCreationFn>  {
						alloc::boxed::Box::new(move || alloc::boxed::Box::new(Self {#(#arguments: #arguments.clone()),*}))
					}
				}
			} else {
				quote! {
					/// Behavior creation function
					#[inline]
					fn create_fn(#(#parameter),*) -> alloc::boxed::Box<behaviortree::behavior::BehaviorCreationFn>  {
						alloc::boxed::Box::new(move || alloc::boxed::Box::new(Self {#(#arguments: #arguments.clone()),*, ..Default::default()}))
					}
				}
			}
		}
	};

	let fn_register = if no_register {
		quote! {}
	} else {
		quote! {
			/// Registers the behavior.
			pub fn register(factory: &mut behaviortree::factory::BehaviorTreeFactory, name: &str) -> Result<(), behaviortree::factory::error::Error> {
				let bhvr_desc = behaviortree::behavior::behavior_description::BehaviorDescription::new(name, name, #kind_token, #groot2, Self::provided_ports());
				let bhvr_creation_fn = Self::create_fn(#(#types::default()),*);
				factory.registry_mut()
					.add_behavior(bhvr_desc, bhvr_creation_fn)
			}
		}
	};

	// dbg!(no_register_with, no_params, &parameter);
	let fn_register_with = if no_register_with || no_params {
		quote! {}
	} else {
		quote! {
			/// Registers the behavior with parameter.
			pub fn register_with(factory: &mut behaviortree::factory::BehaviorTreeFactory, name: &str, #(#parameter),*) -> Result<(), behaviortree::factory::error::Error> {
				let bhvr_desc = behaviortree::behavior::behavior_description::BehaviorDescription::new(name, name, #kind_token, #groot2, Self::provided_ports());
				let bhvr_creation_fn = Self::create_fn(#(#arguments),*);
				factory.registry_mut()
					.add_behavior(bhvr_desc, bhvr_creation_fn)
			}
		}
	};

	// create the impl block only if it has content
	let impl_block = if no_create && no_register && (no_register_with || no_params) {
		quote! {}
	} else {
		quote! {
			#derived
			#diagnostic
			impl #impl_generics #ident #type_generics #where_clause {
				#fn_create_fn
				#fn_register
				#fn_register_with
			}
		}
	};

	Ok(quote! {
		#derived
		#diagnostic
		impl #impl_generics behaviortree::behavior::BehaviorExecution for #ident #type_generics #where_clause {
			#[inline]
			fn as_any(&self) -> &dyn core::any::Any { self }
			#[inline]
			fn as_any_mut(&mut self) -> &mut dyn core::any::Any { self }
			fn creation_fn() -> alloc::boxed::Box<behaviortree::behavior::BehaviorCreationFn> {
				alloc::boxed::Box::new(|| alloc::boxed::Box::new(Self::default()))
			}
			#[inline]
			fn kind() -> behaviortree::behavior::BehaviorKind { #kind_token }
			#[inline]
			fn static_provided_ports(&self) -> behaviortree::port::PortList { Self::provided_ports() }
		}

		#impl_block
	})
}
