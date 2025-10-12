// Copyright © 2025 Stephan Kunz
//! [`Sleep`] [`Action`] implementation.

#[cfg(feature = "std")]
extern crate std;

// region:      --- modules
use crate::{
	self as behaviortree, Action, EMPTY_STR,
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorResult, BehaviorState},
	input_port,
	port::PortList,
	port_list,
	tree::BehaviorTreeElementList,
};
use alloc::{boxed::Box, string::ToString};
use tinyscript::SharedRuntime;

#[cfg(feature = "std")]
use core::time::Duration;
#[cfg(feature = "std")]
use std::time::Instant;
//endregion:    --- modules

// region:		--- globals
/// Port name literals
const MSEC: &str = "msec";
// endregion:	--- globals

// region:		--- Sleep
/// The [`Sleep`] behavior sleeps for the amount of time given via port msec.
/// Consider also using the decorator [`Delay`](crate::behavior::decorator::Delay)
///
/// The behavior is gated behind feature `sleep`.
#[derive(Action, Debug, Default)]
pub struct Sleep {
	#[cfg(feature = "std")]
	start_time: Option<Instant>,
}

#[async_trait::async_trait]
impl Behavior for Sleep {
	#[inline]
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		#[cfg(feature = "std")]
		{
			self.start_time = None;
		}
		Ok(())
	}

	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		#[cfg(feature = "std")]
		{
			self.start_time = Some(Instant::now());
		}
		behavior.set_state(BehaviorState::Running);
		Ok(())
	}

	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let millis: u64 = behavior.get(MSEC)?;
		#[cfg(feature = "std")]
		if let Some(start) = &self.start_time {
			if Instant::now().duration_since(*start) > Duration::from_millis(millis) {
				self.start_time = None;
				Ok(BehaviorState::Success)
			} else {
				Ok(BehaviorState::Running)
			}
		} else {
			Err(BehaviorError::Composition {
				txt: "Sleep has no start_time set".into(),
			})
		}

		#[cfg(not(feature = "std"))]
		{
			let _ = millis;
			Ok(BehaviorState::Success)
		}
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(
			u64,
			MSEC,
			EMPTY_STR,
			"Time to sleep in [msec]."
		)]
	}
}
// endregion:	--- Sleep
