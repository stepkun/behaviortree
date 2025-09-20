// Copyright © 2025 Stephan Kunz
//! [`Repeat`] [`Decorator`] implementation.

// region:      --- modules
use alloc::{boxed::Box, string::ToString};
use tinyscript::SharedRuntime;

use crate::{
	self as behaviortree, Decorator, IDLE,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState, error::BehaviorError},
	input_port,
	port::PortList,
	port_list,
	tree::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:		--- globals
/// Port name literals
const NUM_CYCLES: &str = "num_cycles";
// endregion:	--- globals

// region:      --- Repeat
/// The [`Repeat`] decorator is used to execute a child several times as long as it succeeds.
///
/// Example:
///
/// ```xml
/// <Repeat num_cycles="3">
///     <WaveHand/>
/// </Repeat>
/// ```
#[derive(Decorator, Debug, Default)]
pub struct Repeat {
	/// Defaults to `0`
	repeat_count: i32,
}

#[async_trait::async_trait]
impl Behavior for Repeat {
	#[inline]
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		self.repeat_count = 0;
		Ok(())
	}

	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut ConstBehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		let num_cycles = behavior.get::<i32>(NUM_CYCLES)?;
		if self.repeat_count < num_cycles || num_cycles == -1 {
			let child = &mut children[0];
			let new_state = child.tick(runtime).await?;

			match new_state {
				BehaviorState::Failure => {
					self.repeat_count = 0;
					children.halt(runtime)?;
					Ok(BehaviorState::Failure)
				}
				BehaviorState::Idle => Err(BehaviorError::State("Repeat".into(), IDLE.into())),
				BehaviorState::Running => return Ok(BehaviorState::Running),
				BehaviorState::Skipped => {
					children.halt(runtime)?;
					Ok(BehaviorState::Skipped)
				}
				BehaviorState::Success => {
					self.repeat_count += 1;
					children.halt(runtime)?;
					Ok(BehaviorState::Running)
				}
			}
		} else {
			Ok(BehaviorState::Success)
		}
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(
			i32,
			NUM_CYCLES,
			-1,
			"Repeat a successful child up to N times. Use -1 to create an infinite loop."
		)]
	}
}
// endregion:   --- RetryUntilSuccessful
