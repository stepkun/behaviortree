// Copyright © 2025 Stephan Kunz

//! [`behaviortree`](crate) factory module.

pub mod error;
#[allow(clippy::module_inception)]
mod factory;
pub mod registry;

// flatten
pub use factory::BehaviorTreeFactory;
pub use registry::BehaviorRegistry;
