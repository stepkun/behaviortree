// Copyright © 2025 Stephan Kunz

//! Derive macro `Behavior` for `behavior`
//!

#[doc(hidden)]
extern crate proc_macro;

#[doc(hidden)]
extern crate alloc;

use quote::quote;

/// Derive macro `Behavior`.
#[proc_macro_derive(ScriptEnum, attributes(dimas))]
pub fn derive_behavior(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote!{}.into()
}
