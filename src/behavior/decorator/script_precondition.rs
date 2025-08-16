// Copyright © 2025 Stephan Kunz

//! `ScriptPrecondition` behavior implementation
//!

// region:      --- modules
use alloc::{
	boxed::Box,
	string::{String, ToString},
};
use tinyscript::SharedRuntime;

use crate::{self as behaviortree, EMPTY_STR};
use crate::{
	Decorator, ELSE, IF,
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorResult, BehaviorState},
	input_port,
	port::PortList,
	port_list,
	tree::tree_element_list::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- Precondition
/// The `Precondition` behavior is used to check a scripted condition before
/// executing its child.
#[derive(Decorator, Default)]
pub struct Precondition;

#[async_trait::async_trait]
impl Behavior for Precondition {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut ConstBehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		let if_branch = behavior.get::<String>(IF)?;
		let value = runtime
			.lock()
			.run(&if_branch, behavior.blackboard_mut())?;

		let new_state = if value.is_bool() {
			let val = value.as_bool()?;
			let child = &mut children[0];
			if val {
				// tick child and return the resulting value
				child.tick(runtime).await?
			} else {
				// halt eventually running child
				child.halt_children(runtime)?;
				let else_branch = behavior.get::<String>(ELSE)?;
				match else_branch.as_ref() {
					"Failure" => BehaviorState::Failure,
					"Idle" => BehaviorState::Idle,
					"Running" => BehaviorState::Running,
					"Skipped" => BehaviorState::Skipped,
					"Success" => BehaviorState::Success,
					_ => {
						let value = runtime
							.lock()
							.run(&else_branch, behavior.blackboard_mut())?;
						if value.is_bool() {
							let val = value.as_bool()?;
							if val { BehaviorState::Success } else { BehaviorState::Failure }
						} else {
							return Err(BehaviorError::NotABool);
						}
					}
				}
			}
		} else {
			return Err(BehaviorError::NotABool);
		};

		Ok(new_state)
	}

	fn provided_ports() -> PortList {
		port_list![
			input_port!(String, IF, EMPTY_STR, "Condition to check."),
			input_port!(String, ELSE, EMPTY_STR, "Return state if condition is false."),
		]
	}
}
// endregion:   --- Precondition
