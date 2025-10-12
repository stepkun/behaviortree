// Copyright © 2025 Stephan Kunz
//! [`ForceState`] [`Decorator`] implementation.

// region:      --- modules
use crate::{
	self as behaviortree, Decorator,
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorResult, BehaviorState},
	tree::BehaviorTreeElementList,
};
use alloc::boxed::Box;
use tinyscript::SharedRuntime;
// endregion:   --- modules

// region:      --- ForceState
/// The `ForceState` behavior is used to return a certain state, independant of what the child returned.
/// - If child returns Failure or Success, this behavior returns the stored [`BehaviorState`].
/// - If child returns any other state, that state will be returned.
///
/// This [`Decorator`] is used to provide the decorators that enforce a certain response, independant from the childs result like
/// `ForceFailure` and `ForceSuccess`.
///
/// The behavior is gated behind feature `force_state`.
/// There are the predefined variants
/// - ForceFailure: gated behind feature `force_failure`
/// - ForceRunning: gated behind feature `force_running`
/// - ForceSuccess: gated behind feature `force_success`
#[derive(Decorator, Debug, Default)]
pub struct ForceState {
	state: BehaviorState,
}

#[async_trait::async_trait]
impl Behavior for ForceState {
	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		let child = &mut children[0];
		let new_state = child.tick(runtime).await?;

		match new_state {
			BehaviorState::Failure | BehaviorState::Success => {
				children.halt(runtime)?;
				Ok(self.state)
			}
			BehaviorState::Idle => Err(BehaviorError::State {
				behavior: "ForceState".into(),
				state: new_state,
			}),
			state @ (BehaviorState::Running | BehaviorState::Skipped) => Ok(state),
		}
	}
}

impl ForceState {
	/// Constructor with arguments.
	#[must_use]
	pub const fn new(state: BehaviorState) -> Self {
		Self { state }
	}

	/// Initialization function.
	pub const fn initialize(&mut self, state: BehaviorState) {
		self.state = state;
	}
}
// endregion:   --- ForceState
