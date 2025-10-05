// Copyright © 2025 Stephan Kunz
//! [`ReactiveSequence`] [`Control`] implementation.

// region:      --- modules
use crate::{
	self as behaviortree, Control,
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorResult, BehaviorState},
	tree::BehaviorTreeElementList,
};
use alloc::boxed::Box;
use tinyscript::SharedRuntime;
// endregion:   --- modules

// region:      --- ReactiveSequence
/// A [`ReactiveSequence`] ticks its children in an ordered sequence from first to last at every tick.
/// - If a child returns [`BehaviorState::Failure`] the sequence returns [`BehaviorState::Failure`].
/// - If a child returns [`BehaviorState::Running`] halt the remaining siblings in the sequence
///   and return [`BehaviorState::Running`].
/// - If a child returns [`BehaviorState::Success`], tick the next child.
/// - If all children return [`BehaviorState::Success`] the sequence returns [`BehaviorState::Success`].
///
/// If all the children return [`BehaviorState::Success`], this node returns [`BehaviorState::Success`].
///
/// IMPORTANT: Having asynchronous children (aka children that return [`BehaviorState::Running`]) makes
/// this behavior difficult to predict. Avoid having more than one asynchronous children!
///
/// Example:
///
/// Requires a factory at least `with_core_behaviors` or manual registration
/// ```xml
/// <ReacitveSequence>
///    <Behavior1/>
///    <Behavior2/>
///    <Behavior3/>
/// </ReactiveSequence>
/// ```
#[derive(Control, Debug)]
pub struct ReactiveSequence {
	/// Defaults to '-1'
	running_child_idx: i32,
}

impl Default for ReactiveSequence {
	fn default() -> Self {
		Self { running_child_idx: -1 }
	}
}

#[async_trait::async_trait]
impl Behavior for ReactiveSequence {
	#[inline]
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		self.running_child_idx = -1;
		Ok(())
	}

	#[inline]
	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		behavior.set_state(BehaviorState::Running);
		Ok(())
	}

	#[allow(clippy::cast_possible_truncation)]
	#[allow(clippy::cast_possible_wrap)]
	#[allow(clippy::cast_sign_loss)]
	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		let mut all_skipped = true;

		let children_count = children.len();
		for child_idx in 0..children_count {
			let child = &mut children[child_idx];
			let new_state = child.tick(runtime).await?;

			all_skipped &= new_state == BehaviorState::Skipped;

			match new_state {
				BehaviorState::Failure => {
					self.running_child_idx = -1;
					children.halt(runtime)?;
					return Ok(BehaviorState::Failure);
				}
				BehaviorState::Idle => {
					return Err(BehaviorError::State {
						behavior: "ReactiveSequence".into(),
						state: new_state,
					});
				}
				BehaviorState::Running => {
					// halt previously running child
					if self.running_child_idx != (child_idx as i32) && self.running_child_idx != -1 {
						children[self.running_child_idx as usize].halt_children(runtime)?;
						self.running_child_idx = -1;
					}
					if self.running_child_idx == -1 {
						self.running_child_idx = child_idx as i32;
					} else if self.running_child_idx != (child_idx as i32) {
						// Multiple children running at the same time
						return Err(BehaviorError::Composition {
							txt: "[ReactiveSequence]: Only a single child can return Running.".into(),
						});
					}
					return Ok(BehaviorState::Running);
				}
				BehaviorState::Skipped => {
					// halt current child
					child.halt_children(runtime)?;
					self.running_child_idx = -1;
				}
				BehaviorState::Success => {
					self.running_child_idx = -1;
				}
			}
		}

		// Reset children
		children.halt(runtime)?;

		if all_skipped {
			Ok(BehaviorState::Skipped)
		} else {
			Ok(BehaviorState::Success)
		}
	}
}
// endregion:   --- ReactiveSequence
