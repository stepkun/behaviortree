// Copyright © 2025 Stephan Kunz

//! Built in scripted condition behavior of `DiMAS`

// region:      --- modules
use alloc::{boxed::Box, string::String};
use tinyscript::SharedRuntime;

use crate as behaviortree;
use crate::behavior::BehaviorData;
use crate::{
	Behavior,
	behavior::{BehaviorInstance, BehaviorKind, BehaviorResult, BehaviorState, BehaviorStatic},
	input_port,
	port::PortList,
	port_list,
	tree::ConstBehaviorTreeElementList,
};
//endregion:    --- modules

/// The `ScriptCondition` behavior returns Success or Failure depending on the result of the scripted code.
#[derive(Behavior, Debug, Default)]
pub struct ScriptCondition;

#[async_trait::async_trait]
impl BehaviorInstance for ScriptCondition {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		let code = behavior.get::<String>("code")?;
		let value = runtime
			.lock()
			.run(&code, behavior.blackboard_mut())?;

		let state = if value.is_bool() {
			let val = value.as_bool()?;
			if val { BehaviorState::Success } else { BehaviorState::Failure }
		} else {
			BehaviorState::Failure
		};

		Ok(state)
	}
}

impl BehaviorStatic for ScriptCondition {
	fn kind() -> BehaviorKind {
		BehaviorKind::Condition
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(
			String,
			"code",
			"",
			"Piece of code that can be parsed. Must return false or true."
		)]
	}
}
