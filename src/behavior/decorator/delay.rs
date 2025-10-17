// Copyright © 2025 Stephan Kunz
//! [`Delay`] [`Decorator`] implementation.

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
const DELAY_MSEC: &str = "delay_msec";
// endregion:	--- globals

// region:		--- Delay
/// The [`Delay`] decorator will introduce a delay given by the port `delay_msec` and then tick its child.
/// Consider also using the action [`Sleep`](crate::behavior::action::Sleep)
///
/// The behavior is gated behind feature `delay`.
#[derive(Decorator, Debug, Default)]
#[behavior(groot2)]
pub struct Delay {
	#[cfg(feature = "std")]
	start_time: Option<Instant>,
}

#[async_trait::async_trait]
impl Behavior for Delay {
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
		let millis: u64 = behavior.get(DELAY_MSEC)?;

		#[cfg(not(feature = "std"))]
		{
			let _ = children;
			let _ = runtime;
			let _ = millis;
			Ok(BehaviorState::Success)
		}
		#[cfg(feature = "std")]
		if let Some(start) = &self.start_time {
			if Instant::now().duration_since(*start) > Duration::from_millis(millis) {
				let state = children[0].tick(runtime).await?;
				if state.is_completed() {
					children.halt(runtime)?;
					Ok(BehaviorState::Success)
				} else {
					Ok(state)
				}
			} else {
				Ok(BehaviorState::Running)
			}
		} else {
			Err(BehaviorError::Composition {
				txt: "Delay has no start_time set".into(),
			})
		}
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(
			u64,
			DELAY_MSEC,
			EMPTY_STR,
			"Tick the child after a few milliseconds."
		)]
	}
}
// endregion:	--- Delay
