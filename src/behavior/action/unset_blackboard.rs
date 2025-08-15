// Copyright © 2025 Stephan Kunz

//! `SetBlackboard` behavior implementation
//!

// region:      --- modules
use alloc::string::String;
use alloc::{boxed::Box, string::ToString};
use core::marker::PhantomData;
use core::str::FromStr;
use tinyscript::SharedRuntime;

use crate as behaviortree;
use crate::{
	Action, KEY,
	behavior::{BehaviorData, Behavior, BehaviorResult, BehaviorState},
	port::{PortList, strip_bb_pointer},
	tree::tree_element_list::ConstBehaviorTreeElementList,
};
use crate::{input_port, port_list};
// endregion:   --- modules

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
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let key = behavior.get::<String>(KEY)?;
		match strip_bb_pointer(&key) {
			Some(stripped_key) => {
				let _ = behavior.delete::<String>(&stripped_key);
			}
			None => {
				let _ = behavior.delete::<String>(&key);
			}
		}

		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(
			String,
			KEY,
			"",
			"Key of the entry to remove"
		),]
	}
}
// endregion:   --- UnsetBlackboard
