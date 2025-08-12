// Copyright © 2025 Stephan Kunz

//! Built in [`Sleep`] action behavior

// region:      --- modules
use alloc::boxed::Box;
#[cfg(feature = "std")]
use core::time::Duration;
use tinyscript::SharedRuntime;
#[cfg(feature = "std")]
use tokio::task::JoinHandle;

use crate as behaviortree;
use crate::{
	Action, MSEC,
	behavior::{BehaviorData, BehaviorError, BehaviorInstance, BehaviorResult, BehaviorState, BehaviorStatic},
	input_port,
	port::PortList,
	port_list,
	tree::tree_element_list::ConstBehaviorTreeElementList,
};
//endregion:    --- modules

// region:		--- Sleep
/// The [`Sleep`] behavior sleeps for the amount of time given via port msec.
/// Consider also using the decorator [`Delay`](crate::behavior::decorator::Delay)
#[derive(Action, Debug, Default)]
pub struct Sleep {
	#[cfg(feature = "std")]
	handle: Option<JoinHandle<()>>,
}

#[async_trait::async_trait]
impl BehaviorInstance for Sleep {
	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		#[cfg(not(feature = "std"))]
		let _ = behavior;
		#[cfg(not(feature = "std"))]
		let _ = MSEC;
		#[cfg(feature = "std")]
		let millis: u64 = behavior.get(MSEC)?;
		#[cfg(feature = "std")]
		{
			self.handle = Some(tokio::task::spawn(async move {
				tokio::time::sleep(Duration::from_millis(millis)).await;
			}));
			behavior.set_state(BehaviorState::Running);
			Ok(())
		}
		#[cfg(not(feature = "std"))]
		{
			todo!();
		}
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
}

impl BehaviorStatic for Sleep {
	fn provided_ports() -> PortList {
		port_list![input_port!(
			u64,
			MSEC,
			"",
			"Time to sleep in [msec]."
		)]
	}
}
// endregion:	--- Sleep
