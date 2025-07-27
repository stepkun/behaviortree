// Copyright © 2025 Stephan Kunz

//! Tests the factory

use behaviortree::factory::BehaviorTreeFactory;

#[test]
fn factory_creation() {
	BehaviorTreeFactory::default();
	assert!(BehaviorTreeFactory::with_core_behaviors().is_ok());
	assert!(BehaviorTreeFactory::with_extended_behaviors().is_ok());
	assert!(BehaviorTreeFactory::with_groot2_behaviors().is_ok());
}
