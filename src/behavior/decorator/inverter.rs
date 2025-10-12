// Copyright © 2025 Stephan Kunz
//! [`Inverter`] [`Decorator`] implementation.

// region:      --- modules
use crate::{
	self as behaviortree, Decorator,
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorResult, BehaviorState},
	tree::BehaviorTreeElementList,
};
use alloc::boxed::Box;
use tinyscript::SharedRuntime;
// endregion:   --- modules

// region:      --- Inverter
/// The `Inverter` behavior is used invert the childs outcome:
/// - If child returns Success, this behavior returns Failure.
/// - If child returns Failure, this behavior returns Success.
/// - If child returns Skipped or Running, this state will be returned.
///
/// The behavior is gated behind feature `inverter`.
#[derive(Decorator, Default)]
pub struct Inverter;

#[async_trait::async_trait]
impl Behavior for Inverter {
	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		let child = &mut children[0];
		let new_state = child.tick(runtime).await?;

		match new_state {
			BehaviorState::Failure => {
				children.halt(runtime)?;
				Ok(BehaviorState::Success)
			}
			BehaviorState::Idle => Err(BehaviorError::State {
				behavior: "Inverter".into(),
				state: new_state,
			}),
			state @ (BehaviorState::Running | BehaviorState::Skipped) => Ok(state),
			BehaviorState::Success => {
				children.halt(runtime)?;
				Ok(BehaviorState::Failure)
			}
		}
	}
}
// endregion:   --- Inverter
