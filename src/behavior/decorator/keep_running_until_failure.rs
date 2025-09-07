// Copyright © 2025 Stephan Kunz
//! [`KeepRunningUntilFailure`] [`Decorator`] implementation.

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate::{
	self as behaviortree, Decorator,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState, error::BehaviorError},
	tree::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- KeepRunningUntilFailure
/// The `KeepRunningUntilFailure` decorator is used to execute a child repeatedly until it fails.
///
///
/// Example:
///
/// ```xml
/// <KeepRunningUntilFailure>
///     <OpenDoor/>
/// </KeepRunningUntilFailure>
/// ```
#[derive(Decorator, Default)]
pub struct KeepRunningUntilFailure;

#[async_trait::async_trait]
impl Behavior for KeepRunningUntilFailure {
	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		children: &mut ConstBehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		match children[0].tick(runtime).await? {
			BehaviorState::Failure => {
				children.halt(runtime)?;
				Ok(BehaviorState::Failure)
			}
			BehaviorState::Idle => Err(BehaviorError::Composition(
				"KeepRunningUntilFailure should never return 'Idle'".into(),
			)),
			BehaviorState::Running => Ok(BehaviorState::Running),
			BehaviorState::Skipped => Err(BehaviorError::Composition(
				"KeepRunningUntilFailure should never return 'Skipped'".into(),
			)),
			BehaviorState::Success => {
				children.halt(runtime)?;
				Ok(BehaviorState::Running)
			}
		}
	}
}
// endregion:   --- KeepRunningUntilFailure
