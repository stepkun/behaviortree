// Copyright © 2025 Stephan Kunz

//! Test correct usage of behavior derive macros

use behaviortree::prelude::*;

#[derive(Action, Debug, Default)]
struct TestAction;

#[async_trait::async_trait]
impl Behavior for TestAction {
	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		Ok(BehaviorState::Success)
	}
}

#[derive(Condition, Debug, Default)]
struct TestCondition;

#[async_trait::async_trait]
impl Behavior for TestCondition {
	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		Ok(BehaviorState::Success)
	}
}

#[derive(Control, Debug, Default)]
struct TestControl;

#[async_trait::async_trait]
impl Behavior for TestControl {
	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		Ok(BehaviorState::Success)
	}
}

#[derive(Decorator, Debug, Default)]
struct TestDecorator;

#[async_trait::async_trait]
impl Behavior for TestDecorator {
	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		Ok(BehaviorState::Success)
	}
}

// dummy main
fn main() {}
