// Copyright © 2025 Stephan Kunz
//! [`SetBlackboard`] [`Action`] implementation.

// region:      --- modules
use crate::{
	self as behaviortree, Action, EMPTY_STR,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState},
	inout_port, input_port,
	port::PortList,
	port_list, strip_curly_brackets,
	tree::ConstBehaviorTreeElementList,
};
use alloc::{boxed::Box, string::String, string::ToString};
use core::{fmt::Debug, marker::PhantomData, str::FromStr};
use tinyscript::SharedRuntime;
// endregion:   --- modules

// region:		--- globals
/// Port name literals
const OUTPUT_KEY: &str = "output_key";
const VALUE: &str = "value";
// endregion:	--- globals

// region:      --- SetBlackboard
/// The [`SetBlackboard`] behavior is used to store a value of type T
/// into an entry of the Blackboard specified via port `output_key`.
///
#[derive(Action, Default)]
pub struct SetBlackboard<T: Debug>
where
	T: Clone + Default + FromStr + ToString + Send + Sync + 'static,
{
	_marker: PhantomData<T>,
}

#[async_trait::async_trait]
impl<T> Behavior for SetBlackboard<T>
where
	T: Clone + Debug + Default + FromStr + ToString + Send + Sync,
{
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let value = behavior.get::<T>(VALUE)?;
		let key = behavior.get::<String>(OUTPUT_KEY)?;
		let stripped_key = strip_curly_brackets(&key);
		behavior.set(stripped_key, value)?;

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
