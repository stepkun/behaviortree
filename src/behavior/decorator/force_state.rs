// Copyright © 2025 Stephan Kunz
//! [`ForceState`] [`Decorator`] implementation.

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate::{
	self as behaviortree, Decorator, IDLE,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState, error::BehaviorError},
	tree::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- ForceState
/// The `ForceState` behavior is used to return a certain state, independant of what the child returned.
/// - If child returns Failure or Success, this behavior returns the stored [`BehaviorState`].
/// - If child returns any other state, that state will be returned.
///
/// This [`Decorator`] is used to provide the decorators that enforce a certain response, independant from the childs result like
/// `ForceFailure` and `ForceSuccess`.
/// The registration is not possible via the providedd functions or macros, butmust be done manually using its new(...) method like:
/// ```no-test
/// let bhvr_desc = BehaviorDescription::new(
///     "ForceSkipped",
///     "ForceSkipped",
///     ForceState::kind(),
///     false,
///     ForceState::provided_ports(),
/// );
/// let bhvr_creation_fn =
///     Box::new(move || -> Box<dyn BehaviorExecution> { Box::new(ForceState::new(BehaviorState::Skipped)) });
/// self.registry_mut()
///     add_behavior(bhvr_desc, bhvr_creation_fn)?;
/// ```
#[derive(Decorator, Debug, Default)]
pub struct ForceState {
	state: BehaviorState,
}

#[async_trait::async_trait]
impl Behavior for ForceState {
	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		children: &mut ConstBehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		let child = &mut children[0];
		let new_state = child.tick(runtime).await?;

		match new_state {
			BehaviorState::Failure | BehaviorState::Success => {
				children.halt(runtime)?;
				Ok(self.state)
			}
			BehaviorState::Idle => Err(BehaviorError::State("ForceState".into(), IDLE.into())),
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
