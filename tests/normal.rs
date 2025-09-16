// Copyright © 2025 Stephan Kunz

//! Tests

use behaviortree::{
	BehaviorTree, BehaviorTreeElement, BehaviorTreeElementList,
	port::{PortDefinition, PortList},
};

// check, that the auto traits are available
const fn is_normal<T: Sized + Send + Sync>() {}

#[test]
const fn normal_types() {
	is_normal::<BehaviorTree>();
	is_normal::<BehaviorTreeElementList>();
	is_normal::<BehaviorTreeElement>();

	is_normal::<PortDefinition>();
	is_normal::<PortList>();
}
