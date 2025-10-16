// Copyright © 2025 Stephan Kunz
//! [`RunOnce`] [`Decorator`] implementation.

// region:      --- modules
use crate::{
	self as behaviortree, Decorator,
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorResult, BehaviorState},
	input_port,
	port::PortList,
	port_list,
	tree::BehaviorTreeElementList,
};
use alloc::{boxed::Box, string::ToString};
use tinyscript::SharedRuntime;
// endregion:   --- modules

// region:		--- globals
/// Port name literals
const THEN_SKIP: &str = "then_skip";
// endregion:	--- globals

// region:      --- RunOnce
/// The [`RunOnce`] decorator ticks its child exactly once and returns the state.
/// Afterwards, if `ţhen_skip` is set to `true` Skipped will be returned, otherwise the state of the first run.
///
/// The behavior is gated behind feature `run_once`.
#[derive(Decorator, Debug, Default)]
#[behavior(groot2 = true)]
pub struct RunOnce {
	already_ticked: bool,
	state: BehaviorState,
}

#[async_trait::async_trait]
impl Behavior for RunOnce {
	#[inline]
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		self.already_ticked = false;
		self.state = BehaviorState::Idle;
		Ok(())
	}

	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		if self.already_ticked {
			if behavior.get::<bool>(THEN_SKIP).unwrap_or(true) {
				Ok(BehaviorState::Skipped)
			} else {
				Ok(self.state)
			}
		} else {
			let state = children[0].tick(runtime).await?;
			if state.is_completed() {
				self.already_ticked = true;
				self.state = state;
			} else if state == BehaviorState::Idle {
				return Err(BehaviorError::State {
					behavior: "RunOnce".into(),
					state,
				});
			}
			Ok(state)
		}
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(
			bool,
			THEN_SKIP,
			"true",
			"If true, skip after the first execution, otherwise return the same 'BehaviorState' returned once by the child"
		)]
	}
}
// endregion:   --- RetryUntilSuccessful
