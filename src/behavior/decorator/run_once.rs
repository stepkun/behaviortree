// Copyright © 2025 Stephan Kunz

//! `RunOnce` behavior implementation
//!

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate as behaviortree;
use crate::{
	Decorator, IDLE, THEN_SKIP,
	behavior::{BehaviorData, BehaviorError, BehaviorInstance, BehaviorResult, BehaviorState, BehaviorStatic},
	input_port,
	port::PortList,
	port_list,
	tree::tree_element_list::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- RunOnce
/// The [`RunOnce`] decorator ticks its child exactly once and returns the state.
/// Afterwards, if `ţhen_skip` is set to `true` Skipped will be returned, otherwise the state of the first run.
#[derive(Decorator, Debug, Default)]
pub struct RunOnce {
	already_ticked: bool,
	then_skip: bool,
	state: BehaviorState,
}

#[async_trait::async_trait]
impl BehaviorInstance for RunOnce {
	#[inline]
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		self.already_ticked = false;
		self.state = BehaviorState::Idle;
		Ok(())
	}

	#[inline]
	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		self.then_skip = behavior.get::<bool>(THEN_SKIP)?;
		Ok(())
	}

	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		children: &mut ConstBehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		if self.already_ticked {
			if self.then_skip {
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
				Err(BehaviorError::State("RunOnce".into(), IDLE.into()))?;
			}
			Ok(state)
		}
	}
}

impl BehaviorStatic for RunOnce {
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
