// Copyright © 2025 Stephan Kunz
//! [`UnsetBlackboard`] [`Action`] implementation.

// region:      --- modules
use crate::{
	self as behaviortree, Action, EMPTY_STR,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState},
	input_port,
	port::PortList,
	port_list,
	tree::BehaviorTreeElementList,
};
use alloc::string::String;
use alloc::{boxed::Box, string::ToString};
use core::marker::PhantomData;
use core::str::FromStr;
use tinyscript::SharedRuntime;
// endregion:   --- modules

// region:		--- globals
/// Port name literals
const KEY: &str = "key";
// endregion:	--- globals

// region:      --- UnsetBlackboard
/// The [`UnsetBlackboard`] behavior is used to delete a value of type T
/// from the Blackboard specified via port `key`.
/// Will return Success whether the entry exists or not.
#[derive(Action, Default)]
pub struct UnsetBlackboard<T>
where
	T: Clone + Default + FromStr + ToString + Send + Sync + 'static,
{
	_marker: PhantomData<T>,
}

#[async_trait::async_trait]
impl<T> Behavior for UnsetBlackboard<T>
where
	T: Clone + Default + FromStr + ToString + Send + Sync,
{
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let key = behavior.get::<String>(KEY)?;
		behavior.delete::<String>(&key)?;

		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(
			String,
			KEY,
			EMPTY_STR,
			"Key of the entry to remove"
		),]
	}
}
// endregion:   --- UnsetBlackboard
