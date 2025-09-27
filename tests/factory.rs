// Copyright © 2025 Stephan Kunz

//! Tests the factory

use behaviortree::factory::{BehaviorTreeFactory, error::Error};

#[test]
fn factory_creation() -> Result<(), Error> {
	assert!(BehaviorTreeFactory::new().is_ok());
	assert!(BehaviorTreeFactory::with_core_behaviors().is_ok());
	assert!(BehaviorTreeFactory::with_extended_behaviors().is_ok());
	assert!(BehaviorTreeFactory::with_groot2_behaviors().is_ok());
	assert!(BehaviorTreeFactory::with_all_behaviors().is_ok());

	let mut factory = BehaviorTreeFactory::new()?;
	assert!(factory.register_test_behaviors().is_ok());
	Ok(())
}
