// Copyright © 2025 Stephan Kunz
//! [`Precondition`] [`Decorator`] implementation.

// region:      --- modules
use alloc::{
	boxed::Box,
	string::{String, ToString},
};
use tinyscript::SharedRuntime;

use crate::{
	self as behaviortree, Decorator, EMPTY_STR, FAILURE, IDLE, RUNNING, SKIPPED, SUCCESS,
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorResult, BehaviorState},
	input_port,
	port::PortList,
	port_list,
	tree::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:		--- globals
/// Port name literals
const ELSE: &str = "else";
const IF: &str = "if";
// endregion:	--- globals

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
		let value = runtime.lock().run(&if_branch, behavior)?;

		let new_state = /* if value.is_bool()*/ {
			let val = bool::try_from(value)?;
			let child = &mut children[0];
			if val {
				// tick child and return the resulting value
				child.tick(runtime).await?
			} else {
				// halt eventually running child
				child.halt_children(runtime)?;
				let else_branch = behavior.get::<String>(ELSE)?;
				match else_branch.as_ref() {
					FAILURE => BehaviorState::Failure,
					IDLE => BehaviorState::Idle,
					RUNNING => BehaviorState::Running,
					SKIPPED => BehaviorState::Skipped,
					SUCCESS => BehaviorState::Success,
					_ => {
						let value = runtime
							.lock()
							.run(&else_branch, behavior)?;
						if value.is_bool() {
							let val = bool::try_from(value)?;
							if val { BehaviorState::Success } else { BehaviorState::Failure }
						} else {
							return Err(BehaviorError::NotABool);
						}
					}
				}
			}
		// } else {
		// 	return Err(BehaviorError::NotABool);
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
