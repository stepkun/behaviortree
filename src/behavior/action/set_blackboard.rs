// Copyright © 2025 Stephan Kunz

//! `SetBlackboard` behavior implementation
//!

// region:      --- modules
use alloc::string::String;
use alloc::{boxed::Box, string::ToString};
use core::marker::PhantomData;
use core::str::FromStr;
use tinyscript::SharedRuntime;

use crate::{self as behaviortree, EMPTY_STR};
use crate::{
	Action, OUTPUT_KEY, VALUE,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState},
	port::{PortList, strip_bb_pointer},
	tree::tree_element_list::ConstBehaviorTreeElementList,
};
use crate::{inout_port, input_port, port_list};
// endregion:   --- modules

// region:      --- SetBlackboard
/// The [`SetBlackboard`] behavior is used to store a value of type T
/// into an entry of the Blackboard specified via port `output_key`.
///
#[derive(Action, Default)]
pub struct SetBlackboard<T>
where
	T: Clone + Default + FromStr + ToString + Send + Sync + 'static,
{
	_marker: PhantomData<T>,
}

#[async_trait::async_trait]
impl<T> Behavior for SetBlackboard<T>
where
	T: Clone + Default + FromStr + ToString + Send + Sync,
{
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let value = behavior.get::<T>(VALUE)?;
		let key = behavior.get::<String>(OUTPUT_KEY)?;
		match strip_bb_pointer(&key) {
			Some(stripped_key) => {
				behavior.set(&stripped_key, value)?;
			}
			None => {
				behavior.set(&key, value)?;
			}
		}

		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list![
			input_port!(T, VALUE, EMPTY_STR, "Value to be written into the output_key"),
			inout_port!(
				String,
				OUTPUT_KEY,
				EMPTY_STR,
				"Name of the blackboard entry where the value should be written"
			),
		]
	}
}
// endregion:   --- SetBlackboard
