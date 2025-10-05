// Copyright © 2025 Stephan Kunz
//! [`Fallback`] & `AsyncFallback` [`Control`] implementations.

use crate::{
	self as behaviortree, Control,
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorResult, BehaviorState},
	tree::BehaviorTreeElementList,
};
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

/// The `Fallback` behavior is used to try different strategies until one succeeds.
/// If any child returns [`BehaviorState::Running`], previous children will NOT be ticked again.
/// - If all the children return [`BehaviorState::Failure`], this node returns [`BehaviorState::Failure`].
/// - If a child returns [`BehaviorState::Running`], this node returns [`BehaviorState::Running`].
/// - If a child returns [`BehaviorState::Success`] this behavior returns [`BehaviorState::Success`].
///
/// It implements 2 modes, which differ in how they handle a childs failure:
/// - The synchronous mode will tick all children within one tick from its parent.
/// - The asynchronous mode will return the flow contol after a childs failure to its parent
///   returning [`BehaviorState::Running`] and continue with the next child at the next tick from parent.
///
/// While running, the loop is not restarted, first the running child will be ticked again.
/// If that tick fails the sequence continues, children that already failed will not be ticked again.
///
/// Examples:
///
/// ```xml
/// <Fallback>
///    <Behavior1/>
///    <Behavior2/>
///    <Behavior3/>
/// </Fallback>
/// ```
///
/// Requires a factory at least `with_core_behaviors` or manual registration
/// ```xml
/// <AsyncFallback>
///    <Behavior1/>
///    <Behavior2/>
///    <Behavior3/>
/// </AsyncFallback>
/// ```
#[derive(Control, Debug, Default)]
pub struct Fallback {
	/// Defaults to '0'
	child_idx: usize,
	/// Defaults to '0'
	skipped: usize,
	/// Asynchronous mode flag
	asynch: bool,
}

#[async_trait::async_trait]
impl Behavior for Fallback {
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
			let prev_state = child.state();
			let child_state = child.tick(runtime).await?;

			match child_state {
				BehaviorState::Failure => {
					self.child_idx += 1;
					if self.asynch && (prev_state == BehaviorState::Idle) && (self.child_idx < children_count) {
						return Ok(BehaviorState::Running);
					}
				}
				BehaviorState::Idle => {
					return Err(BehaviorError::State {
						behavior: "Fallback".into(),
						state: child_state,
					});
				}
				BehaviorState::Running => return Ok(child_state),
				BehaviorState::Success => {
					children.reset(runtime)?;
					children.halt(runtime)?;
					self.child_idx = 0;
					return Ok(child_state);
				}
				BehaviorState::Skipped => {
					self.child_idx += 1;
					self.skipped += 1;
				}
			}
		}

		// loop ended without a success,
		// so either all children failed or were skipped
		let all_skipped = self.skipped == children_count;
		if self.child_idx >= children_count {
			children.reset(runtime)?;
			self.child_idx = 0;
			self.skipped = 0;
		}
		if all_skipped {
			Ok(BehaviorState::Skipped)
		} else {
			Ok(BehaviorState::Failure)
		}
	}
}

impl Fallback {
	/// Returns a Fallback behavior with the given asynchronouisity.
	#[must_use]
	pub const fn new(asynch: bool) -> Self {
		Self {
			child_idx: 0,
			skipped: 0,
			asynch,
		}
	}
}
