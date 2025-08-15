// Copyright © 2025 Stephan Kunz

//! [`behaviortree`](crate) factory module.

pub mod behavior_registry;
pub mod error;
#[allow(clippy::module_inception)]
mod factory;

// flatten
pub use behavior_registry::BehaviorRegistry;
pub use factory::BehaviorTreeFactory;
