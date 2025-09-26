// Copyright © 2025 Stephan Kunz

//! [`behaviortree`](crate) factory module.

pub mod error;
#[allow(clippy::module_inception)]
mod factory;
pub mod registry;

// flatten
pub use factory::BehaviorTreeFactory;
pub use registry::BehaviorRegistry;

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<error::Error>();
		is_normal::<BehaviorTreeFactory>();
		is_normal::<BehaviorRegistry>();
	}
}
