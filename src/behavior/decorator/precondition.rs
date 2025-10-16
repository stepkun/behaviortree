// Copyright © 2025 Stephan Kunz
//! [`Precondition`] [`Decorator`] implementation.

// region:      --- modules
use crate::{
	self as behaviortree, Decorator, EMPTY_STR,
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorResult, BehaviorState},
	input_port,
	port::PortList,
	port_list,
	tree::BehaviorTreeElementList,
};
use alloc::{
	boxed::Box,
	string::{String, ToString},
};
use tinyscript::SharedRuntime;
// endregion:   --- modules

// region:		--- globals
/// Port name literals
const ELSE: &str = "else";
const IF: &str = "if";
// endregion:	--- globals

// region:      --- Precondition
/// The `Precondition` behavior is used to check a scripted condition before
/// executing its child.
///
/// The behavior is gated behind feature `precondition`.
#[derive(Decorator, Default)]
#[behavior(groot2 = true)]
pub struct Precondition;

#[async_trait::async_trait]
impl Behavior for Precondition {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		let if_branch = behavior.get::<String>(IF)?;
		let value = runtime.lock().run(&if_branch, behavior)?;

		let new_state = {
			let val = bool::try_from(value)?;
			let child = &mut children[0];
			if val {
				// tick child and return the resulting value
				child.tick(runtime).await?
			} else {
				// halt eventually running child
				child.halt_children(runtime)?;
				let else_branch = behavior.get::<String>(ELSE)?.to_uppercase();

				match else_branch.as_ref() {
					"FAILURE" => BehaviorState::Failure,
					"IDLE" => BehaviorState::Idle,
					"RUNNING" => BehaviorState::Running,
					"SKIPPED" => BehaviorState::Skipped,
					"SUCCESS" => BehaviorState::Success,
					_ => {
						let value = runtime.lock().run(&else_branch, behavior)?;
						if value.is_bool() {
							let val = bool::try_from(value)?;
							if val { BehaviorState::Success } else { BehaviorState::Failure }
						} else {
							return Err(BehaviorError::NotABool {
								value: value.to_string().into(),
							});
						}
					}
				}
			}
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
