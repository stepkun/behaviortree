// Copyright © 2025 Stephan Kunz
//! [`ScriptCondition`] [`Condition`] implementation.

// region:      --- modules
use alloc::{
	boxed::Box,
	string::{String, ToString},
};
use tinyscript::SharedRuntime;

use crate::{
	self as behaviortree, Condition, EMPTY_STR,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState},
	input_port,
	port::PortList,
	port_list,
	tree::BehaviorTreeElementList,
};
//endregion:    --- modules

// region:		--- globals
/// Port name literals
const CODE: &str = "code";
// endregion:	--- globals

/// The `ScriptCondition` behavior returns Success or Failure depending on the result of the scripted code.
#[derive(Condition, Default)]
pub struct ScriptCondition;

#[async_trait::async_trait]
impl Behavior for ScriptCondition {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		let code = behavior.get::<String>(CODE)?;
		let value = runtime.lock().run(&code, behavior)?;

		let state = if value.is_bool() {
			let val = bool::try_from(value)?;
			if val { BehaviorState::Success } else { BehaviorState::Failure }
		} else {
			BehaviorState::Failure
		};

		Ok(state)
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(
			String,
			CODE,
			EMPTY_STR,
			"Piece of code that can be parsed. Must return false or true."
		)]
	}
}
