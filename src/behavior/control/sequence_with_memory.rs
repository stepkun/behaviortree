// Copyright © 2025 Stephan Kunz
//! [`SequenceWithMemory`] [`Control`] implementation.

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate::{
	self as behaviortree, Control, IDLE,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState, error::BehaviorError},
	tree::tree_element_list::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- SequenceWithMemory
/// A `SequenceWithMemory` ticks its children in an ordered sequence from first to last.
/// If any child returns RUNNING, previous children are not ticked again.
/// - If all the children return SUCCESS, this node returns SUCCESS.
/// - If a child returns RUNNING, this node returns RUNNING.
///   Loop is NOT restarted, the same running child will be ticked again.
/// - If a child returns FAILURE, stop the loop and return FAILURE.
///
///   Loop is NOT restarted, the same running child will be ticked again.
#[derive(Control, Debug)]
pub struct SequenceWithMemory {
	/// Defaults to '0'
	child_idx: usize,
	/// Defaults to 'true'
	all_skipped: bool,
}

impl Default for SequenceWithMemory {
	fn default() -> Self {
		Self {
			child_idx: 0,
			all_skipped: true,
		}
	}
}

#[async_trait::async_trait]
impl Behavior for SequenceWithMemory {
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
					// Do NOT reset children on failure
					// Halt children at and after current index
					children.halt_from(self.child_idx, runtime)?;
					return Ok(BehaviorState::Failure);
				}
				BehaviorState::Idle => {
					return Err(BehaviorError::State("SequenceWithMemory".into(), IDLE.into()));
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
// endregion:   --- SequenceWithMemory
