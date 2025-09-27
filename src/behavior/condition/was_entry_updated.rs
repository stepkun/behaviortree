// Copyright © 2025 Stephan Kunz
//! [`WasEntryUpdated`] [`Condition`] implementation.

// region:      --- modules
use crate::{
	self as behaviortree, Condition, ConstString, EMPTY_STR,
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
const ENTRY: &str = "entry";
// endregion:	--- globals

// region:      --- WasEntryUpdated
/// The `WasEntryUpdated` condition returns Success if a blackboard entry was updated otherwise Failure.
/// # Errors
/// - if the entry does not exist
#[derive(Condition, Debug, Default)]
pub struct WasEntryUpdated {
	/// ID of the last checked update
	sequence_id: usize,
	/// The entry to monitor
	entry_key: ConstString,
}

#[async_trait::async_trait]
impl Behavior for WasEntryUpdated {
	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		self.sequence_id = 0;
		if let Some(key) = behavior.remappings().find(ENTRY) {
			self.entry_key = key;
			// match strip_bb_pointer(&key) {
			// 	Some(stripped) => self.entry_key = behavior.get::<String>(&stripped)?.into(),
			// 	None => self.entry_key = key,
			// }
			Ok(())
		} else {
			Err(BehaviorError::PortNotDeclared {
				port: "entry".into(),
				behavior: behavior.description().name().clone(),
			})
		}
	}

	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let sequence_id = behavior.sequence_id(&self.entry_key)?;
		if sequence_id == self.sequence_id {
			Ok(BehaviorState::Failure)
		} else {
			self.sequence_id = sequence_id;
			Ok(BehaviorState::Success)
		}
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(
			String,
			ENTRY,
			EMPTY_STR,
			"The blackboard entry to check."
		)]
	}
}
// endregion:   --- WasEntryUpdated
