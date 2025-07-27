// Copyright © 2025 Stephan Kunz

//! A [`BehaviorTreeObserver`] library
//!

pub mod groot2_connector;
pub mod groot2_protocol;
#[allow(clippy::module_inception)]
mod tree_observer;

// flatten
pub use tree_observer::BehaviorTreeObserver;
