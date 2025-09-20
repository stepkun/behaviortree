// Copyright © 2025 Stephan Kunz
//! [`Sleep`] [`Action`] implementation.

// region:      --- modules
use alloc::{boxed::Box, string::ToString};
use tinyscript::SharedRuntime;

use crate::{
	self as behaviortree, Action, EMPTY_STR,
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorResult, BehaviorState},
	input_port,
	port::PortList,
	port_list,
	tree::ConstBehaviorTreeElementList,
};

#[cfg(feature = "std")]
use core::time::Duration;
#[cfg(feature = "std")]
use tokio::task::JoinHandle;
//endregion:    --- modules

// region:		--- globals
/// Port name literals
const MSEC: &str = "msec";
// endregion:	--- globals

// region:		--- Sleep
/// The [`Sleep`] behavior sleeps for the amount of time given via port msec.
/// Consider also using the decorator [`Delay`](crate::behavior::decorator::Delay)
#[derive(Action, Debug, Default)]
pub struct Sleep {
	#[cfg(feature = "std")]
	handle: Option<JoinHandle<()>>,
}

#[async_trait::async_trait]
impl Behavior for Sleep {
	#[inline]
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		#[cfg(feature = "std")]
		{
			self.handle = None;
		}
		Ok(())
	}

	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		let millis: u64 = behavior.get(MSEC)?;
		#[cfg(not(feature = "std"))]
		{
			let _ = millis;
		}
		#[cfg(feature = "std")]
		{
			self.handle = Some(tokio::task::spawn(async move {
				tokio::time::sleep(Duration::from_millis(millis)).await;
			}));
		}
		behavior.set_state(BehaviorState::Running);
		Ok(())
	}

	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		#[cfg(feature = "std")]
		if let Some(handle) = self.handle.as_ref() {
			if handle.is_finished() {
				self.handle = None;
				Ok(BehaviorState::Success)
			} else {
				Ok(BehaviorState::Running)
			}
		} else {
			Ok(BehaviorState::Failure)
		}

		#[cfg(not(feature = "std"))]
		Ok(BehaviorState::Failure)
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
