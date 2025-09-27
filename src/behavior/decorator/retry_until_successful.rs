// Copyright © 2025 Stephan Kunz
//! [`RetryUntilSuccessful`] [`Decorator`] implementation.

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
const NUM_ATTEMPTS: &str = "num_attempts";
// endregion:	--- globals

// region:      --- RetryUntilSuccessful
/// The `RetryUntilSuccessful` decorator is used to execute a child several times if it fails.
///
/// If the child returns Success, the loop is stopped and this decorator
/// returns Success.
///
/// If the child returns Failure, this decorator will try again up to N times
/// (N is read from port `num_attempts`).
///
/// This decorator is non-reactive and does all attempts within 1 tick.
///
/// Example:
///
/// ```xml
/// <RetryUntilSuccessful num_attempts="3">
///     <OpenDoor/>
/// </RetryUntilSuccessful>
/// ```
#[derive(Decorator, Debug, Default)]
pub struct RetryUntilSuccessful;

#[async_trait::async_trait]
impl Behavior for RetryUntilSuccessful {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		let max_attempts = behavior.get::<i32>(NUM_ATTEMPTS).unwrap_or(-1);
		let mut try_count = 0;
		let mut all_skipped = true;

		while try_count < max_attempts || max_attempts == -1 {
			// A `Decorator` has only 1 child
			let child = &mut children[0];
			let new_state = child.tick(runtime).await?;

			all_skipped &= new_state == BehaviorState::Skipped;

			match new_state {
				BehaviorState::Failure => {
					try_count += 1;
					children.halt(runtime)?;
				}
				BehaviorState::Idle => {
					return Err(BehaviorError::State {
						behavior: "RetryUntilSuccessful".into(),
						state: new_state,
					});
				}
				BehaviorState::Running => return Ok(BehaviorState::Running),
				BehaviorState::Skipped => {
					children.halt(runtime)?;
					return Ok(BehaviorState::Skipped);
				}
				BehaviorState::Success => {
					children.halt(runtime)?;
					return Ok(BehaviorState::Success);
				}
			}
		}

		if all_skipped {
			Ok(BehaviorState::Skipped)
		} else {
			Ok(BehaviorState::Failure)
		}
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(
			i32,
			NUM_ATTEMPTS,
			-1,
			"Try up to N times. Use -1 to create an infinite loop."
		)]
	}
}
// endregion:   --- RetryUntilSuccessful
