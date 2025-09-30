// Copyright © 2025 Stephan Kunz
//! [`Sequence`] & `AsyncSequence` [`Control`] implementations.

use crate::{
	self as behaviortree, Control,
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorResult, BehaviorState},
	tree::BehaviorTreeElementList,
};
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

/// A `Sequence` ticks its children in an ordered sequence from first to last.
/// If any child returns [`BehaviorState::Running`], previous children will NOT be ticked again.
/// - If any child returns [`BehaviorState::Failure`] the sequence returns [`BehaviorState::Failure`].
/// - If all children return [`BehaviorState::Success`] the sequence returns [`BehaviorState::Success`].
/// - While any child returns [`BehaviorState::Running`] the sequence returns [`BehaviorState::Running`].
///
/// It implements 2 modes, which differ in how they handle a childs success:
/// - The synchronous mode will tick all children within one tick from its parent.
/// - The asynchronous mode will return the flow contol after a childs success to its parent
///   returning [`BehaviorState::Running`] and continue with the next child at the next tick from parent.
///
/// While running, the loop is not restarted, first the running child will be ticked again.
/// If that tick succeeds the sequence continues, children that already succeeded will not be ticked again.
///
/// Examples:
/// <Sequence>
///    <Behavior1/>
///    <Behavior2/>
///    <Behavior3/>
/// </Sequence>
///
/// Requires a factory at least `with_core_behaviors` or manual registration
/// <AsyncSequence>
///    <Behavior1/>
///    <Behavior2/>
///    <Behavior3/>
/// </AsyncSequence>
#[derive(Control, Debug, Default)]
pub struct Sequence {
	/// Defaults to '0'
	child_idx: usize,
	/// Defaults to '0'
	skipped: usize,
	/// Asynchronous mode flag
	asynch: bool,
}

#[async_trait::async_trait]
impl Behavior for Sequence {
	#[inline]
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		self.child_idx = 0;
		self.skipped = 0;
		Ok(())
	}

	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		if !behavior.is_active() {
			self.skipped = 0;
		}
		behavior.set_state(BehaviorState::Running);

		let children_count = children.len();
		while self.child_idx < children_count {
			let child = &mut children[self.child_idx];
			let prev_state = child.data().state();
			let child_state = child.tick(runtime).await?;

			match child_state {
				BehaviorState::Failure => {
					children.reset(runtime)?;
					self.child_idx = 0;
					return Ok(child_state);
				}
				BehaviorState::Idle => {
					return Err(BehaviorError::State {
						behavior: "Sequence".into(),
						state: child_state,
					});
				}
				BehaviorState::Running => return Ok(child_state),
				BehaviorState::Skipped => {
					self.child_idx += 1;
					self.skipped += 1;
				}
				BehaviorState::Success => {
					self.child_idx += 1;
					if self.asynch && (prev_state == BehaviorState::Idle) && (self.child_idx < children_count) {
						return Ok(BehaviorState::Running);
					}
				}
			}
		}

		// All children returned Success or were skipped
		let all_skipped = self.skipped == children_count;
		if self.child_idx >= children_count {
			children.reset(runtime)?;
			self.child_idx = 0;
			self.skipped = 0;
		}
		if all_skipped {
			Ok(BehaviorState::Skipped)
		} else {
			Ok(BehaviorState::Success)
		}
	}
}

impl Sequence {
	/// Returns a Sequence behavior with the given asynchronouisity.
	#[must_use]
	pub const fn new(asynch: bool) -> Self {
		Self {
			child_idx: 0,
			skipped: 0,
			asynch,
		}
	}
}
