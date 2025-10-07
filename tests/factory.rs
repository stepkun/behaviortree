// Copyright © 2025 Stephan Kunz

//! Tests the factory

use behaviortree::factory::{BehaviorTreeFactory, error::Error};

#[test]
fn factory_creation() -> Result<(), Error> {
	assert!(BehaviorTreeFactory::new().is_ok());

	let _factory = BehaviorTreeFactory::new()?;

	Ok(())
}
