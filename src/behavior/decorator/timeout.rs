// Copyright © 2025 Stephan Kunz
//! [`Timeout`] [`Decorator`] implementation.

// region:      --- modules
use alloc::{boxed::Box, string::ToString};
use tinyscript::SharedRuntime;

use crate::{
	self as behaviortree, Decorator, EMPTY_STR,
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

// region:		--- Timeout
/// The [`Timeout`] decorator will halt its child after a period given by the port `msec`.
#[derive(Decorator, Debug, Default)]
pub struct Timeout {
	#[cfg(feature = "std")]
	handle: Option<JoinHandle<()>>,
}

#[async_trait::async_trait]
impl Behavior for Timeout {
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
		children: &mut ConstBehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		#[cfg(not(feature = "std"))]
		let _ = children;
		#[cfg(not(feature = "std"))]
		let _ = runtime;
		#[cfg(feature = "std")]
		if let Some(handle) = self.handle.as_ref() {
			let state = children[0].tick(runtime).await?;
			if state.is_completed() {
				children.halt(runtime)?;
				Ok(state)
			} else if handle.is_finished() {
				self.handle = None;
				children[0].halt_children(runtime)?;
				Ok(BehaviorState::Failure)
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
			"Timeout the child after a few milliseconds."
		)]
	}
}
// endregion:	--- Delay
