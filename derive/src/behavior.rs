// Copyright © 2025 Stephan Kunz

//! Derive macro [`Behavior`] implementation

#[doc(hidden)]
extern crate proc_macro;

#[doc(hidden)]
extern crate alloc;

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// Implementation of the derive macro [`Behavior`]
pub fn derive_behavior_struct(input: &DeriveInput) -> TokenStream {
    // structure name
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let derived: TokenStream = "#[automatically_derived]"
        .parse()
        .expect("derive(Behavior) - derived");
    let diagnostic: TokenStream = "#[diagnostic::do_not_recommend]"
        .parse()
        .expect("derive(Behavior) - diagnostic");

    quote! {
        #derived
        #diagnostic
        impl #impl_generics behaviortree::behavior::Behavior for #ident #type_generics #where_clause {}

        #derived
        #diagnostic
        impl #impl_generics behaviortree::behavior::BehaviorCreation for #ident #type_generics #where_clause {
            fn creation_fn() -> alloc::boxed::Box<behaviortree::behavior::BehaviorCreationFn> {
                alloc::boxed::Box::new(|| alloc::boxed::Box::new(Self::default()))
            }
        }

        #derived
        #diagnostic
        impl #impl_generics behaviortree::behavior::BehaviorExecution for #ident #type_generics #where_clause {
            fn as_any(&self) -> &dyn core::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn core::any::Any { self }
        }

        #derived
        #diagnostic
        impl #impl_generics behaviortree::behavior::BehaviorRedirection for #ident #type_generics #where_clause {
            fn static_provided_ports(&self) -> behaviortree::port::PortList {
                Self::provided_ports()
            }
        }
    }
}
