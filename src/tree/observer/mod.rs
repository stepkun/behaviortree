// Copyright © 2025 Stephan Kunz

//! A [`BehaviorTreeObserver`] library
//!

#[cfg(feature = "std")]
pub mod groot2_connector;
#[cfg(feature = "std")]
pub mod groot2_protocol;
#[allow(clippy::module_inception)]
pub mod tree_observer;

// flatten
