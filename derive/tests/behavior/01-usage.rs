// Copyright © 2025 Stephan Kunz

//! Test correct usage of behavior derive macros 

use behaviortree::prelude::*;

#[derive(Action, Debug, Default)]
struct TestAction;

#[async_trait::async_trait]
impl BehaviorInstance for TestAction {
	async fn tick(
		&mut self,
		_behavior: &mut behaviortree::behavior::BehaviorData,
		_children: &mut behaviortree::tree::ConstBehaviorTreeElementList,
		_runtime: &tinyscript::runtime::SharedRuntime,
	) -> behaviortree::behavior::BehaviorResult {
        Ok(behaviortree::behavior::BehaviorState::Success)
    }
}

impl BehaviorStatic for TestAction {}


#[derive(Condition, Debug, Default)]
struct TestCondition;

#[async_trait::async_trait]
impl BehaviorInstance for TestCondition {
	async fn tick(
		&mut self,
		_behavior: &mut behaviortree::behavior::BehaviorData,
		_children: &mut behaviortree::tree::ConstBehaviorTreeElementList,
		_runtime: &tinyscript::runtime::SharedRuntime,
	) -> behaviortree::behavior::BehaviorResult {
        Ok(behaviortree::behavior::BehaviorState::Success)
    }
}

impl BehaviorStatic for TestCondition {}

#[derive(Control, Debug, Default)]
struct TestControl;

#[async_trait::async_trait]
impl BehaviorInstance for TestControl {
	async fn tick(
		&mut self,
		_behavior: &mut behaviortree::behavior::BehaviorData,
		_children: &mut behaviortree::tree::ConstBehaviorTreeElementList,
		_runtime: &tinyscript::runtime::SharedRuntime,
	) -> behaviortree::behavior::BehaviorResult {
        Ok(behaviortree::behavior::BehaviorState::Success)
    }
}

impl BehaviorStatic for TestControl {}

#[derive(Decorator, Debug, Default)]
struct TestDecorator;

#[async_trait::async_trait]
impl BehaviorInstance for TestDecorator {
	async fn tick(
		&mut self,
		_behavior: &mut behaviortree::behavior::BehaviorData,
		_children: &mut behaviortree::tree::ConstBehaviorTreeElementList,
		_runtime: &tinyscript::runtime::SharedRuntime,
	) -> behaviortree::behavior::BehaviorResult {
        Ok(behaviortree::behavior::BehaviorState::Success)
    }
}

impl BehaviorStatic for TestDecorator {}

// dummy main
fn main(){}
