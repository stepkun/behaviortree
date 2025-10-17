// Copyright © 2025 Stephan Kunz
//! [`Timeout`] [`Decorator`] implementation.

#[cfg(feature = "std")]
extern crate std;

// region:      --- modules
use crate::{
	self as behaviortree, Decorator, EMPTY_STR,
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

// region:		--- Timeout
/// The [`Timeout`] decorator will halt its child after a period given by the port `msec`.
///
/// The behavior is gated behind feature `timeout`.
#[derive(Decorator, Debug, Default)]
#[behavior(groot2)]
pub struct Timeout {
	#[cfg(feature = "std")]
	start_time: Option<Instant>,
}

#[async_trait::async_trait]
impl Behavior for Timeout {
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
		children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		let millis: u64 = behavior.get(MSEC)?;
		#[cfg(not(feature = "std"))]
		{
			let _ = millis;
			let _ = children;
			let _ = runtime;
			Ok(BehaviorState::Failure)
		}
		#[cfg(feature = "std")]
		if let Some(start) = &self.start_time {
			let state = children[0].tick(runtime).await?;
			if state.is_completed() {
				self.start_time = None;
				children.halt(runtime)?;
				Ok(state)
			} else if Instant::now().duration_since(*start) > Duration::from_millis(millis) {
				children[0].halt_children(runtime)?;
				self.start_time = None;
				Ok(BehaviorState::Failure)
			} else {
				Ok(BehaviorState::Running)
			}
		} else {
			Err(BehaviorError::Composition {
				txt: "Timeout has no start_time set".into(),
			})
		}
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
