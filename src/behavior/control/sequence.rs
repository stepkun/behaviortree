// Copyright © 2025 Stephan Kunz
//! [`Sequence`] [`Control`] implementation.

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate::{
	self as behaviortree, Control, IDLE,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState, error::BehaviorError},
	tree::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- Sequence
/// A `Sequence` ticks its children in an ordered sequence from first to last.
/// - If any child returns [`BehaviorState::Failure`] the sequence returns [`BehaviorState::Failure`].
/// - If all children return [`BehaviorState::Success`] the sequence returns [`BehaviorState::Success`].
/// - While any child returns [`BehaviorState::Running`] the sequence returns [`BehaviorState::Running`].
///
/// While running, the loop is not restarted, first the running child will be ticked again.
/// If that tick succeeds the sequence continues, children that already succeeded will not be ticked again.
#[derive(Control, Debug)]
pub struct Sequence {
	/// Defaults to '0'
	child_idx: usize,
	/// Defaults to 'true'
	all_skipped: bool,
}

impl Default for Sequence {
	fn default() -> Self {
		Self {
			child_idx: 0,
			all_skipped: true,
		}
	}
}

#[async_trait::async_trait]
impl Behavior for Sequence {
	#[inline]
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		self.child_idx = 0;
		self.all_skipped = true;
		Ok(())
	}

	#[inline]
	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		behavior.set_state(BehaviorState::Running);
		Ok(())
	}

	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		children: &mut ConstBehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		while self.child_idx < children.len() {
			let child = &mut children[self.child_idx];
			let new_state = child.tick(runtime).await?;

			self.all_skipped &= new_state == BehaviorState::Skipped;

			match new_state {
				BehaviorState::Failure => {
					children.halt(runtime)?;
					self.child_idx = 0;
					return Ok(BehaviorState::Failure);
				}
				BehaviorState::Idle => {
					return Err(BehaviorError::State("Sequence".into(), IDLE.into()));
				}
				BehaviorState::Running => return Ok(BehaviorState::Running),
				BehaviorState::Skipped | BehaviorState::Success => {
					self.child_idx += 1;
				}
			}
		}

		// All children returned Success
		if self.child_idx >= children.len() {
			// Reset children
			children.halt(runtime)?;
			self.child_idx = 0;
		}

		if self.all_skipped {
			Ok(BehaviorState::Skipped)
		} else {
			Ok(BehaviorState::Success)
		}
	}
}
// endregion:   --- Sequence
