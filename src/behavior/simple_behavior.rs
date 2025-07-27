// Copyright © 2025 Stephan Kunz

//! `DiMAS` implementation for registering functions as behavior

// region:      --- modules
use alloc::{boxed::Box, sync::Arc};
use core::any::Any;
use tinyscript::SharedRuntime;

use crate::{behavior::BehaviorData, port::PortList, tree::ConstBehaviorTreeElementList};

use super::{BehaviorCreationFn, BehaviorExecution, BehaviorInstance, BehaviorRedirection, BehaviorResult};
// endregion:   --- modules

// region:      --- types
/// Signature of a simple registered behavior function called by `SimpleBehavior`'s tick
pub type SimpleBhvrTickFn = Arc<dyn Fn() -> BehaviorResult + Send + Sync>;

/// Signature of a registered behavior function called by `SimpleBehavior`'s tick
pub type ComplexBhvrTickFn = Arc<dyn Fn(&mut BehaviorData) -> BehaviorResult + Send + Sync>;
// endregion:   --- types

// region:      --- BehaviorFunction
/// A simple behavior
pub struct SimpleBehavior {
	/// The function to be called on tick
	simple_tick_fn: Option<SimpleBhvrTickFn>,
	/// The function to be called on tick if ports exist
	complex_tick_fn: Option<ComplexBhvrTickFn>,
	/// List of provided ports
	provided_ports: PortList,
}

impl core::fmt::Debug for SimpleBehavior {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("SimpleBehavior")
			//.field("tick_fn", &self.tick_fn)
			.finish()
	}
}

impl BehaviorExecution for SimpleBehavior {
	fn as_any(&self) -> &dyn Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}
}

#[async_trait::async_trait]
impl BehaviorInstance for SimpleBehavior {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		self.complex_tick_fn.as_ref().map_or_else(
			|| {
				self.simple_tick_fn.as_ref().map_or_else(
					|| {
						Err(super::BehaviorError::Composition(
							"SimpleBehavior without tick function".into(),
						))
					},
					|func| func(),
				)
			},
			|func| func(behavior),
		)
	}
}

impl BehaviorRedirection for SimpleBehavior {
	fn static_provided_ports(&self) -> PortList {
		self.provided_ports.clone()
	}
}

//impl BehaviorStaticMethods for SimpleBehavior {}

/// Implementation resembles the macro generated impl code
impl SimpleBehavior {
	/// Create a `SimpleBehavior` with the given function
	pub fn create(tick_fn: SimpleBhvrTickFn) -> Box<BehaviorCreationFn> {
		Box::new(move || {
			Box::new(Self {
				simple_tick_fn: Some(tick_fn.clone()),
				complex_tick_fn: None,
				provided_ports: PortList::default(),
			})
		})
	}

	/// Create a `SimpleBehavior` with the given function and list of ports
	pub fn new_create_with_ports(tick_fn: ComplexBhvrTickFn, port_list: PortList) -> Box<BehaviorCreationFn> {
		Box::new(move || {
			Box::new(Self {
				simple_tick_fn: None,
				complex_tick_fn: Some(tick_fn.clone()),
				provided_ports: port_list.clone(),
			})
		})
	}
}
// endregion:   --- BehaviorFunction
