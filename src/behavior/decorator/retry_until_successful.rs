// Copyright © 2025 Stephan Kunz

//! `RetryUntilSuccessful` behavior implementation
//!

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate as behaviortree;
use crate::{
	Decorator, IDLE,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState, error::BehaviorError},
	input_port,
	port::PortList,
	port_list,
	tree::tree_element_list::ConstBehaviorTreeElementList,
};
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
#[derive(Decorator, Debug)]
pub struct RetryUntilSuccessful {
	/// Defaults to `-1`
	max_attempts: i32,
	/// Defaults to `0`
	try_count: i32,
	/// Defaults to `true`
	all_skipped: bool,
}

impl Default for RetryUntilSuccessful {
	fn default() -> Self {
		Self {
			max_attempts: -1,
			try_count: 0,
			all_skipped: true,
		}
	}
}

#[async_trait::async_trait]
impl Behavior for RetryUntilSuccessful {
	#[inline]
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		self.try_count = 0;
		self.all_skipped = true;
		Ok(())
	}

	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		// Load num_cycles from the port value
		self.max_attempts = behavior.get::<i32>(NUM_ATTEMPTS)?;
		behavior.set_state(BehaviorState::Running);
		Ok(())
	}

	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		children: &mut ConstBehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		while self.try_count < self.max_attempts || self.max_attempts == -1 {
			// A `Decorator` has only 1 child
			let child = &mut children[0];
			let new_state = child.tick(runtime).await?;

			self.all_skipped &= new_state == BehaviorState::Skipped;

			match new_state {
				BehaviorState::Failure => {
					self.try_count += 1;
					children.halt(runtime)?;
				}
				BehaviorState::Idle => {
					return Err(BehaviorError::State("RetryUntilSuccessful".into(), IDLE.into()));
				}
				BehaviorState::Running => return Ok(BehaviorState::Running),
				BehaviorState::Skipped => {
					children.halt(runtime)?;
					return Ok(BehaviorState::Skipped);
				}
				BehaviorState::Success => {
					children.halt(runtime)?;
					self.try_count = 0;
					return Ok(BehaviorState::Success);
				}
			}
		}

		if self.all_skipped {
			Ok(BehaviorState::Skipped)
		} else {
			Ok(BehaviorState::Failure)
		}
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(i32, NUM_ATTEMPTS)]
	}
}
// endregion:   --- RetryUntilSuccessful
