// Copyright © 2025 Stephan Kunz

//! Common derive macro implementation

#[doc(hidden)]
extern crate proc_macro;

#[doc(hidden)]
extern crate alloc;

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// Implementation of the derive macro
pub fn derive_behavior_struct(input: &DeriveInput, kind: super::Kind) -> TokenStream {
	// structure name
	let ident = &input.ident;
	let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

	let derived: TokenStream = "#[automatically_derived]"
		.parse()
		.expect("derive(Behavior) - derived");
	let diagnostic: TokenStream = "#[diagnostic::do_not_recommend]"
		.parse()
		.expect("derive(Behavior) - diagnostic");

	let kind_ = match kind {
		crate::Kind::Action => quote! { behaviortree::behavior::BehaviorKind::Action },
		crate::Kind::Condition => quote! { behaviortree::behavior::BehaviorKind::Condition },
		crate::Kind::Control => quote! { behaviortree::behavior::BehaviorKind::Control },
		crate::Kind::Decorator => quote! { behaviortree::behavior::BehaviorKind::Decorator },
	};

	quote! {
		#derived
		#diagnostic
		impl #impl_generics behaviortree::behavior::Behavior for #ident #type_generics #where_clause {
			fn creation_fn() -> alloc::boxed::Box<behaviortree::behavior::BehaviorCreationFn> {
				alloc::boxed::Box::new(|| alloc::boxed::Box::new(Self::default()))
			}
			#[inline]
			fn kind() -> behaviortree::behavior::BehaviorKind { #kind_ }
		}

		#derived
		#diagnostic
		impl #impl_generics behaviortree::behavior::BehaviorExecution for #ident #type_generics #where_clause {
			#[inline]
			fn as_any(&self) -> &dyn core::any::Any { self }
			#[inline]
			fn as_any_mut(&mut self) -> &mut dyn core::any::Any { self }
			#[inline]
			fn static_provided_ports(&self) -> behaviortree::port::PortList { Self::provided_ports() }
		}
	}
}
