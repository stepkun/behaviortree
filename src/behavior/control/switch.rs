// Copyright © 2025 Stephan Kunz

//! `Switch` behavior implementation

// region:      --- modules
use alloc::boxed::Box;
use alloc::string::String;
use tinyscript::SharedRuntime;

use crate::{self as behaviortree, EMPTY_STR};
use crate::{
	CASES, ConstString, Control, IDLE, VARIABLE,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState, error::BehaviorError},
	input_port,
	port::{PortList, is_bb_pointer, strip_bb_pointer},
	tree::tree_element_list::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- Switch
#[allow(rustdoc::invalid_html_tags)]
/// The `Switch` behavior is equivalent to a C/C++ `switch` or a Rust `match` statement,
/// where a certain branch (child) is executed according to the value of a variable.
///
/// Example:
///
/// <Switch3 variable="{var}"  case_1="1" case_2="42" case_3="666" >
///    <ActionA name="action_when_var_eq_1" />
///    <ActionB name="action_when_var_eq_42" />
///    <ActionC name="action_when_var_eq_666" />
///    <ActionD name="default_action" />
///  </Switch3>
///
/// When the Switch behavior is executed (Switch3 is a behavior with 3 cases)
/// the "variable" will be compared to the cases and execute the correct child
/// or the default one (last).
///
/// Note: The same behaviour can be achieved with multiple `Sequences`, `Fallbacks` and `Conditions`,
/// but switch is shorter and therefor more readable.
#[derive(Control, Debug)]
pub struct Switch<const T: u8> {
	/// Defaults to T
	cases: u8,
	/// Defaults to '-1'
	running_child_index: i32,
	/// Defaults to empty
	var: ConstString,
}

impl<const T: u8> Default for Switch<T> {
	fn default() -> Self {
		Self {
			cases: T,
			running_child_index: -1,
			var: EMPTY_STR.into(),
		}
	}
}

#[async_trait::async_trait]
impl<const T: u8> Behavior for Switch<T> {
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		self.cases = T;
		self.running_child_index = -1;
		Ok(())
	}

	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		self.running_child_index = -1;

		// check composition
		if children.len() != (self.cases + 1) as usize {
			return Err(BehaviorError::Composition(
				"Wrong number of children in Switch behavior: must be (num_cases + 1)!".into(),
			));
		}
		if let Some(var) = behavior.remappings.find(&VARIABLE.into()) {
			if is_bb_pointer(&var) {
				if let Some(var) = strip_bb_pointer(&var) {
					self.var = var;
				} else {
					return Err(BehaviorError::Composition(
						"port [variable] must be a Blackboard pointer".into(),
					));
				}
			} else {
				return Err(BehaviorError::Composition(
					"port [variable] must be a Blackboard pointer".into(),
				));
			}
		} else {
			return Err(BehaviorError::Composition("port [variable] must be defined".into()));
		}
		behavior.set_state(BehaviorState::Running);
		Ok(())
	}

	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut ConstBehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		// default match index
		let default_index = i32::from(T);
		let mut match_index = i32::from(T);
		let var = behavior.get::<String>(&self.var)?;
		for i in 0..T {
			let case = behavior.get::<String>(CASES[i as usize])?;

			// string comparison
			if var == case {
				match_index = i32::from(i);
				break;
			}

			// compare as enums
			let guard = runtime.lock();
			if let Some(c_val) = guard.enum_discriminant(&case) {
				if let Ok(v_val) = var.parse::<i8>() {
					if c_val == v_val {
						match_index = i32::from(i);
						break;
					}
				} else if let Some(v_val) = guard.enum_discriminant(&var) {
					if c_val == v_val {
						match_index = i32::from(i);
						break;
					}
				}
			}
			drop(guard);

			// compare as integers
			if let Ok(v_val) = var.parse::<i64>() {
				if let Ok(c_val) = case.parse::<i64>() {
					if c_val == v_val {
						match_index = i32::from(i);
						break;
					}
				}
			}

			// compare as floats
			if let Ok(c_val) = case.parse::<f64>() {
				if let Ok(v_val) = var.parse::<f64>() {
					let delta = f64::abs(v_val - c_val);
					if delta <= 0.000_000_000_000_002 {
						match_index = i32::from(i);
						break;
					}
				}
			}
		}

		// stop child, if it is not the one that should run
		if self.running_child_index > 0 && match_index != self.running_child_index && match_index <= default_index {
			#[allow(clippy::cast_sign_loss)]
			children[self.running_child_index as usize].halt_children(runtime)?;
		}

		#[allow(clippy::cast_sign_loss)]
		let state = children[match_index as usize]
			.tick(runtime)
			.await?;

		if state == BehaviorState::Skipped {
			// if the matching child is Skipped, should default be executed or
			// return just Skipped? Going with the latter for now.
			self.running_child_index = -1;
		} else if state == BehaviorState::Idle {
			return Err(BehaviorError::State("Switch".into(), IDLE.into()));
		} else if state == BehaviorState::Running {
			self.running_child_index = match_index;
		} else {
			children.halt(runtime)?;
			self.running_child_index = -1;
		}
		Ok(state)
	}

	fn provided_ports() -> PortList {
		let mut ports = PortList::default();
		let port = input_port!(String, VARIABLE);
		ports
			.add(port)
			.expect("providing port [variable] failed in behavior [Switch<T>]");

		for i in 0..T {
			let port = input_port!(String, CASES[i as usize]);
			ports
				.add(port)
				.expect("providing port [case_T] failed in behavior [Switch<T>]");
		}
		ports
	}
}
// endregion:   --- Switch
