// Copyright © 2025 Stephan Kunz
//! [`IfThenElse`] [`Control`] implementation.

// region:      --- modules
use crate::{
	self as behaviortree, Control,
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorResult, BehaviorState},
	tree::BehaviorTreeElementList,
};
use alloc::boxed::Box;
use tinyscript::SharedRuntime;
// endregion:   --- modules

// region:      --- IfThenElse
/// The `IfThenElse` behavior must have exactly 2 or 3 children. This behavior is __not__ reactive.
/// A reactive variant is the [`WhileDoElse`](crate::behavior::control::WhileDoElse) behavior.
///
/// The first child is the 'statement' of the if.
/// - If that returns [`BehaviorState::Success`], then the second child is executed until it succeeds or fails.
/// - Instead, if it returns [`BehaviorState::Failure`], the third child is executed until it succeeds or fails.
///
/// If you have only 2 children, this node will return [`BehaviorState::Failure`] whenever the
/// statement returns [`BehaviorState::Failure`].
/// This is equivalent to adding [`AlwaysFailure`](crate::behavior::action::ChangeStateAfter) as 3rd child.
///
/// The behavior is gated behind feature `if_then_else`.
///
/// Example:
///
/// ```xml
/// <IfThenElse>
///    <Condition/>
///    <ThenBehavior/>
///    <ElseBehavior/>
/// </IfThenElse>
/// ```
#[derive(Control, Debug, Default)]
pub struct IfThenElse {
	child_index: usize,
}

#[async_trait::async_trait]
impl Behavior for IfThenElse {
	#[inline]
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		self.child_index = 0;
		Ok(())
	}

	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		// check composition only once at start
		if !(2..=3).contains(&children.len()) {
			return Err(BehaviorError::Composition {
				txt: "IfThenElse must have either 2 or 3 children.".into(),
			});
		}
		behavior.set_state(BehaviorState::Running);
		Ok(())
	}

	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		behavior.set_state(BehaviorState::Running);

		let children_count = children.len();

		if self.child_index == 0 {
			let condition_state = children[0].tick(runtime).await?;
			match condition_state {
				BehaviorState::Failure => match children_count {
					3 => {
						self.child_index = 2;
					}
					2 => {
						return Ok(condition_state);
					}
					_ => {
						return Err(BehaviorError::Composition {
							txt: "Should not happen in 'IfThenElse'".into(),
						});
					}
				},
				BehaviorState::Idle => {
					return Err(BehaviorError::State {
						behavior: "IfThenElse".into(),
						state: condition_state,
					});
				}
				BehaviorState::Running => {
					return Ok(BehaviorState::Running);
				}
				BehaviorState::Skipped => {
					return Ok(BehaviorState::Skipped);
				}
				BehaviorState::Success => {
					self.child_index = 1;
				}
			}
		}

		// execute the branch
		if self.child_index > 0 {
			let state = children[self.child_index].tick(runtime).await?;
			if state != BehaviorState::Running {
				children.reset(runtime)?;
				self.child_index = 0;
			}
			Ok(state)
		} else {
			Err(BehaviorError::Composition {
				txt: "Something unexpected happened in IfThenElse".into(),
			})
		}
	}
}
// endregion:   --- IfThenElse
