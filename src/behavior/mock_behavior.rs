// Copyright © 2025 Stephan Kunz
//! [`MockBehavior`]  implementation.

#[cfg(feature = "std")]
extern crate std;

use super::{Behavior, BehaviorCreationFn, BehaviorExecution, BehaviorResult, BehaviorState};
use crate::{
	BehaviorDescription, BehaviorError, BehaviorKind, BehaviorTreeFactory, ConstString, behavior::BehaviorData,
	port::PortList, tree::BehaviorTreeElementList,
};
use alloc::{boxed::Box, sync::Arc};
use core::{any::Any, time::Duration};
#[cfg(feature = "std")]
use std::time::Instant;
use tinyscript::SharedRuntime;

// region:		--- MockBehaviorConfig
/// Configuration for the [`MockBehavior`].
#[derive(Clone, Default)]
pub struct MockBehaviorConfig {
	/// The [`BehaviorState`] that will be returned finally.
	pub return_state: BehaviorState,
	/// Script to execute when `complete_func()` returns SUCCESS
	pub success_script: Option<ConstString>,
	/// Script to execute when `complete_func()` returns SUCCESS
	pub failure_script: Option<ConstString>,
	/// Script to execute when Behavior is completed
	pub post_script: Option<ConstString>,
	/// If `async_delay` > 0, this behavior becomes asynchronous and
	/// waits this amount of time returning [`BehaviorState::Running`] meanwhile
	pub async_delay: Option<Duration>,
	/// Function invoked when the behavior is completed.
	/// If not specified, the behavior will return `return_state`
	pub complete_func: Option<Arc<dyn Fn() -> BehaviorState + Send + Sync>>,
}

impl core::fmt::Debug for MockBehaviorConfig {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("MockBehaviorConfig")
			.field("return_state", &self.return_state)
			.field("success_script", &self.success_script)
			.field("failure_script", &self.failure_script)
			.field("post_script", &self.post_script)
			.field("async_delay", &self.async_delay)
			// .field("complete_func", &self.complete_func)
			.finish_non_exhaustive()
	}
}

impl MockBehaviorConfig {
	/// Creates a configuration with the geiven return state.
	#[must_use]
	pub fn new(return_state: BehaviorState) -> Self {
		Self {
			return_state,
			..Default::default()
		}
	}
}
// endregion:	--- MockBehaviorConfig

// region:		--- MockBehavior
/// A configurable behavior usable for mocking et. al.
///
/// The behavior is gated behind feature `mock_behavior`.
/// There are the predefined variants
/// - `AlwaysFailure`: gated behind feature `always_failure`
/// - `AlwaysRunning`: gated behind feature `always_running`
/// - `AlwaysSuccess`: gated behind feature `always_success`
#[derive(Default)]
pub struct MockBehavior {
	config: MockBehaviorConfig,
	port_list: PortList,
	#[cfg(feature = "std")]
	start_time: Option<Instant>,
}

impl BehaviorExecution for MockBehavior {
	fn as_any(&self) -> &dyn Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}

	fn creation_fn() -> Box<BehaviorCreationFn> {
		alloc::boxed::Box::new(|| alloc::boxed::Box::new(Self::default()))
	}

	fn kind() -> BehaviorKind {
		BehaviorKind::Action
	}
	fn static_provided_ports(&self) -> PortList {
		self.port_list.clone()
	}
}

#[async_trait::async_trait]
impl Behavior for MockBehavior {
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		#[cfg(feature = "std")]
		{
			self.start_time = None;
		}
		Ok(())
	}

	async fn start(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		if self.config.return_state == BehaviorState::Idle {
			return Err(BehaviorError::Composition {
				txt: "MockBehavior may not return IDLE".into(),
			});
		}
		if self.config.async_delay.is_some() {
			// asynchronous mode
			// remember start time
			#[cfg(feature = "std")]
			{
				self.start_time = Some(Instant::now());
				Ok(BehaviorState::Running)
			}
			#[cfg(not(feature = "std"))]
			{
				self.completed(behavior, runtime)
			}
		} else {
			// synchronous mode
			self.completed(behavior, runtime)
		}
	}

	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		#[cfg(feature = "std")]
		if let Some(delay) = &self.config.async_delay
			&& let Some(start) = &self.start_time
		{
			if Instant::now().duration_since(*start) > *delay {
				self.start_time = None;
				self.completed(behavior, runtime)
			} else {
				Ok(BehaviorState::Running)
			}
		} else {
			self.completed(behavior, runtime)
		}
		#[cfg(not(feature = "std"))]
		self.completed(behavior, runtime)
	}
}

/// Implementation resembles the macro generated impl code
impl MockBehavior {
	/// Creates a `MockBehavior` with the given configuration.
	#[must_use]
	pub const fn new(config: MockBehaviorConfig, port_list: PortList) -> Self {
		Self {
			config,
			port_list,
			#[cfg(feature = "std")]
			start_time: None,
		}
	}

	/// Sets the state field.
	pub const fn set_state(&mut self, state: BehaviorState) {
		self.config.return_state = state;
	}

	/// A `MockBehavior` creation function with the given configuration.
	#[must_use]
	#[allow(clippy::needless_pass_by_value)]
	#[deprecated]
	pub fn creation_fn(config: MockBehaviorConfig, port_list: PortList) -> Box<BehaviorCreationFn> {
		Box::new(move || {
			Box::new(Self {
				config: config.clone(),
				port_list: port_list.clone(),
				#[cfg(feature = "std")]
				start_time: None,
			})
		})
	}

	#[allow(clippy::unnecessary_wraps)]
	/// Returns the result state considering all configuration assets.
	fn completed(&self, behavior: &mut BehaviorData, runtime: &SharedRuntime) -> BehaviorResult {
		let state = self
			.config
			.complete_func
			.as_ref()
			.map_or(self.config.return_state, |func| func());

		// success or failure script set?
		if state == BehaviorState::Success
			&& let Some(script) = &self.config.success_script
		{
			let _result = runtime.lock().run(script, behavior)?;
		} else if state == BehaviorState::Failure
			&& let Some(script) = &self.config.failure_script
		{
			let _result = runtime.lock().run(script, behavior)?;
		}

		// post script set?
		if let Some(script) = &self.config.post_script {
			let _result = runtime.lock().run(script, behavior)?;
		}
		// final result
		Ok(state)
	}

	/// Creates a `creation_fn()` for `MockBehavior` with the given configuration.
	#[must_use]
	#[allow(clippy::needless_pass_by_value)]
	pub fn create_fn(config: MockBehaviorConfig, port_list: PortList) -> Box<BehaviorCreationFn> {
		Box::new(move || {
			Box::new(Self {
				config: config.clone(),
				port_list: port_list.clone(),
				#[cfg(feature = "std")]
				start_time: None,
			})
		})
	}

	/// Registers the `MockBehavior` behavior in the factory.
	/// # Errors
	/// - if registration fails
	pub fn register_with(
		factory: &mut BehaviorTreeFactory,
		name: &str,
		config: MockBehaviorConfig,
		groot2: bool,
	) -> Result<(), crate::factory::error::Error> {
		let bhvr_desc = BehaviorDescription::new(name, name, BehaviorKind::Action, groot2, Self::provided_ports());
		let bhvr_creation_fn = Self::create_fn(config, PortList::default());
		factory
			.registry_mut()
			.add_behavior(bhvr_desc, bhvr_creation_fn)
	}
}
// endregion:	--- MockBehavior
