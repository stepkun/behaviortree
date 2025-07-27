// Copyright © 2025 Stephan Kunz

//! Derive macro [`Behavior`] for `dimas-behavior`
//!

#[doc(hidden)]
extern crate proc_macro;

#[doc(hidden)]
extern crate alloc;

mod behavior;

use behavior::derive_behavior_struct;
use proc_macro::TokenStream;
use syn::DeriveInput;

/// Derive macro for [`Behavior`].
///
/// # Usage
/// ```no_test
/// #[derive(Behavior)]
/// struct MyBehavior {
///     // specific elements
///     ...
/// }
///
/// impl MyBehavior {
///     // specific implementations
///     ...
/// }
/// ```
///
/// # Result
/// Expands the above example to
/// ```no_test
/// struct MyBehavior {
///     // specific elements
///     ...
/// }
///
/// impl MyBehavior {
///     // specific implementations
///     ...
/// }
///
/// #[automatically_derived]
/// #[diagnostic::do_not_recommend]
/// impl behaviortree::behavior::Behavior for MyBehavior {}
///
/// #[automatically_derived]
/// #[diagnostic::do_not_recommend]
/// impl behaviortree::behavior::BehaviorCreation for Fallback {
///     fn creation_fn() -> alloc::boxed::Box<behaviortree::behavior::BehaviorCreationFn> {
///         alloc::boxed::Box::new(|| alloc::boxed::Box::new(Self::default()))
///     }
/// }
///
/// #[automatically_derived]
/// #[diagnostic::do_not_recommend]
/// impl behaviortree::behavior::BehaviorExecution for MyBehavior {
///     fn as_any(&self) -> &dyn core::any::AnyAny { self }
///     fn as_any_mut(&mut self) -> &mut dyn core::any::AnyAny { self }
/// }
///
/// #[automatically_derived]
/// #[diagnostic::do_not_recommend]
/// impl behaviortree::behavior::BehaviorRedirection for MyBehavior {
///     fn static_provided_ports(&self) -> behaviortree::port::PortList {
///         Self::provided_ports()
///     }
/// }
/// ```
///
/// # Errors
///
/// # Panics
/// - if used on enums or unions
#[proc_macro_derive(Behavior, attributes(dimas))]
pub fn derive_behavior(input: TokenStream) -> TokenStream {
	// Construct a representation of the Rust code
	let input: DeriveInput = syn::parse2(input.into()).expect("could not parse input");

	// Check type of input
	match &input.data {
		syn::Data::Struct(_struct) => derive_behavior_struct(&input).into(),
		syn::Data::Enum(_enum) => panic!("enums not supported"),
		syn::Data::Union(_union) => panic!("unions not supported"),
	}
}
