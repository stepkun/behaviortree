// Copyright © 2025 Stephan Kunz
//! [`Inverter`] [`Decorator`] implementation.

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate::{
	self as behaviortree, Decorator, IDLE,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState, error::BehaviorError},
	tree::tree_element_list::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- Inverter
/// The `Inverter` behavior is used invert the childs outcome:
/// - If child returns Success, this behavior returns Failure.
/// - If child returns Failure, this behavior returns Success.
/// - If child returns Skipped or Running, this state will be returned.
#[derive(Decorator, Default)]
pub struct Inverter;

#[async_trait::async_trait]
impl Behavior for Inverter {
	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		children: &mut ConstBehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		let child = &mut children[0];
		let new_state = child.tick(runtime).await?;

		match new_state {
			BehaviorState::Failure => {
				children.halt(runtime)?;
				Ok(BehaviorState::Success)
			}
			BehaviorState::Idle => Err(BehaviorError::State("Inverter".into(), IDLE.into())),
			state @ (BehaviorState::Running | BehaviorState::Skipped) => Ok(state),
			BehaviorState::Success => {
				children.halt(runtime)?;
				Ok(BehaviorState::Failure)
			}
		}
	}
}
// endregion:   --- Inverter
