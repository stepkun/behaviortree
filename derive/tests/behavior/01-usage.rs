// Copyright © 2025 Stephan Kunz

//! Test correct usage of behavior derive macro `Behavior` 

#[doc(hidden)]
extern crate alloc;

use behaviortree::behavior::{BehaviorInstance, BehaviorStatic};

#[derive(behaviortree_derive::Behavior, Debug, Default)]
struct TestBehavior;

#[async_trait::async_trait]
impl BehaviorInstance for TestBehavior {
	async fn tick(
		&mut self,
		_behavior: &mut behaviortree::behavior::BehaviorData,
		_children: &mut behaviortree::tree::ConstBehaviorTreeElementList,
		_runtime: &tinyscript::runtime::SharedRuntime,
	) -> behaviortree::behavior::BehaviorResult {
        Ok(behaviortree::behavior::BehaviorState::Success)
    }
}

impl BehaviorStatic for TestBehavior {
	fn kind() -> behaviortree::behavior::BehaviorKind {
		behaviortree::behavior::BehaviorKind::Action
	}
}

// dummy main
fn main(){}
