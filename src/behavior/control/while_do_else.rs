// Copyright © 2025 Stephan Kunz
//! [`WhileDoElse`] [`Control`] implementation.

// region:      --- modules
use crate::{
	self as behaviortree, Control,
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorResult, BehaviorState},
	tree::BehaviorTreeElementList,
};
use alloc::boxed::Box;
use tinyscript::SharedRuntime;
// endregion:   --- modules

// region:      --- WhileDoElse
/// The `WhileDoElse` behavior must have exactly 2 or 3 children.
/// It is a reactive variant of the [`IfThenElse`](crate::behavior::control::IfThenElse).
///
/// The first child is the 'statement' of the while.
/// - While that returns [`BehaviorState::Success`], then the second child is executed until it succeeds or fails.
/// - Instead, if it returns [`BehaviorState::Failure`], the third child is executed until it succeeds or fails.
///
/// If the second or third child is running and the 'statement' changes its value,
/// the running child is stopped before the sibling is started.
///
/// If you have only 2 children, this node will return [`BehaviorState::Failure`] whenever the
/// statement returns [`BehaviorState::Failure`].
/// This is equivalent to adding [`AlwaysFailure`](crate::behavior::action::ChangeStateAfter) as 3rd child.
///
/// The behavior is gated behind feature `while_do_else`.
///
/// Example:
///
/// ```xml
/// <WhileDoElse>
///    <Condition/>
///    <DoBehavior/>
///    <ElseBehavior/>
/// </WhileDoElse>
/// ```
///
/// Note: This behavior is not a Loop by itself. Use [`Repeat`](crate::behavior::decorator::Repeat) with `-1`
/// for `num_cycles` above the `WhileDoElse` or above the two branches to create individual loopnig behavior.
#[derive(Control, Default)]
#[behavior(groot2)]
pub struct WhileDoElse;

#[async_trait::async_trait]
impl Behavior for WhileDoElse {
	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		// check composition only once at start
		if !(2..=3).contains(&children.len()) {
			return Err(BehaviorError::Composition {
				txt: "WhileDoElse must have either 2 or 3 children.".into(),
			});
		}
		behavior.set_state(BehaviorState::Running);
		Ok(())
	}

	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		let children_count = children.len();

		let condition_status = children[0].tick(runtime).await?;

		if matches!(condition_status, BehaviorState::Running) {
			return Ok(BehaviorState::Running);
		}

		let status = match condition_status {
			BehaviorState::Failure => match children_count {
				3 => {
					children.halt_at(1, runtime)?;
					children[2].tick(runtime).await?
				}
				2 => BehaviorState::Failure,
				_ => {
					return Err(BehaviorError::Composition {
						txt: "Should not happen in 'WhileDoElse'".into(),
					});
				}
			},
			BehaviorState::Idle => {
				return Err(BehaviorError::State {
					behavior: "WhileDoElse".into(),
					state: condition_status,
				});
			}
			BehaviorState::Running => {
				return Ok(BehaviorState::Running);
			}
			BehaviorState::Skipped => {
				return Ok(BehaviorState::Skipped);
			}
			BehaviorState::Success => {
				if children_count == 3 {
					children.halt_at(2, runtime)?;
				}

				children[1].tick(runtime).await?
			}
		};

		match status {
			BehaviorState::Running => Ok(BehaviorState::Running),
			status => {
				children.halt(runtime)?;
				Ok(status)
			}
		}
	}
}
// endregion:   --- WhileDoElse
