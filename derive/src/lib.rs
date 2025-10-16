// Copyright © 2025 Stephan Kunz

//! Derive macro for [`Behavior`](crate)s.
//! There are 4 derive macros avialable:
//! - Action
//! - Condition
//! - Control
//! - Decorator
//!
//! # Usage
//! Using the derive macro Action, the others work respectively.
//! ```no_test
//! #[derive(Action)]
//! struct MyAction {
//!     // specific elements
//!     ...
//! }
//!
//! impl MyAction {
//!     // specific implementations
//!     ...
//! }
//! ```
//!
//! # Result
//! Expands the above example to
//! ```no_test
//! struct MyAction {
//!     // specific elements
//!     ...
//! }
//!
//! impl MyAction {
//!     // specific implementations
//!     ...
//! }
//!
//! #[automatically_derived]
//! #[diagnostic::do_not_recommend]
//! impl behaviortree::behavior::Behavior for MyAction {
//!     fn creation_fn() -> alloc::boxed::Box<behaviortree::behavior::BehaviorCreationFn> {
//!         alloc::boxed::Box::new(|| alloc::boxed::Box::new(Self::default()))
//!     }
//!
//!     fn kind() -> behaviortree::behavior::BehaviorKind {
//!         behaviortree::behavior::BehaviorKind::Action
//!     }
//! }
//!
//! #[automatically_derived]
//! #[diagnostic::do_not_recommend]
//! impl behaviortree::behavior::BehaviorExecution for MyAction {
//!     fn as_any(&self) -> &dyn core::any::Any { self }
//!     fn as_any_mut(&mut self) -> &mut dyn core::any::Any { self }
//!     fn static_provided_ports(&self) -> behaviortree::port::PortList { Self::provided_ports() }
//! }
//! ```
//!
//! # Errors
//!
//! # Panics
//! - if used on enums or unions

#[doc(hidden)]
extern crate proc_macro;

#[doc(hidden)]
extern crate alloc;

mod behavior;

use behavior::derive_behavior_struct;
use proc_macro::TokenStream;

/// internal differantiation of the different kinds of [`Behavior`](crate)s.
enum Kind {
	Action,
	Condition,
	Control,
	Decorator,
}

/// Derive macro for an [`Action`] type [`Behavior`](crate).
#[proc_macro_derive(Action, attributes(behavior))]
pub fn derive_action(input: TokenStream) -> TokenStream {
	derive_behavior_struct(input.into(), Kind::Action)
		.unwrap()
		.into()
}

/// Derive macro for an [`Condition`] type [`Behavior`](crate).
#[proc_macro_derive(Condition, attributes(behavior))]
pub fn derive_condition(input: TokenStream) -> TokenStream {
	derive_behavior_struct(input.into(), Kind::Condition)
		.unwrap()
		.into()
}

/// Derive macro for an [`Control`] type [`Behavior`](crate).
#[proc_macro_derive(Control, attributes(behavior))]
pub fn derive_control(input: TokenStream) -> TokenStream {
	derive_behavior_struct(input.into(), Kind::Control)
		.unwrap()
		.into()
}

/// Derive macro for an [`Decorator`] type [`Behavior`](crate).
#[proc_macro_derive(Decorator, attributes(behavior))]
pub fn derive_decorator(input: TokenStream) -> TokenStream {
	derive_behavior_struct(input.into(), Kind::Decorator)
		.unwrap()
		.into()
}
