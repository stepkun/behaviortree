// Copyright © 2025 Stephan Kunz

//! Built in scripted action behavior

// region:      --- modules
use alloc::{
	boxed::Box,
	string::{String, ToString},
};
use tinyscript::SharedRuntime;

use crate::{self as behaviortree, EMPTY_STR};
use crate::{
	Action, CODE,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState},
	input_port,
	port::PortList,
	port_list,
	tree::tree_element_list::ConstBehaviorTreeElementList,
};
//endregion:    --- modules

/// The `Script` behavior returns Success or Failure depending on the result of the scripted code.
#[derive(Action, Default)]
pub struct Script;

#[async_trait::async_trait]
impl Behavior for Script {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		let code = behavior.get::<String>(CODE)?;
		let value = runtime
			.lock()
			.run(&code, behavior.blackboard_mut())?;

		let state = if value.is_bool() {
			let val = value.as_bool()?;
			if val { BehaviorState::Success } else { BehaviorState::Failure }
		} else {
			BehaviorState::Success
		};

		Ok(state)
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(
			String,
			CODE,
			EMPTY_STR,
			"Piece of code that can be parsed."
		)]
	}
}
