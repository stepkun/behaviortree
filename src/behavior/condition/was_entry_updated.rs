// Copyright © 2025 Stephan Kunz

//! `Updated` behavior implementation
//!

// region:      --- modules
use alloc::boxed::Box;
use alloc::string::String;
use tinyscript::SharedRuntime;

use crate as behaviortree;
use crate::{
	Condition, ConstString, ENTRY,
	behavior::{BehaviorData, BehaviorError, BehaviorInstance, BehaviorResult, BehaviorState, BehaviorStatic},
	port::{PortList, strip_bb_pointer},
	tree::tree_element_list::ConstBehaviorTreeElementList,
};
use crate::{input_port, port_list};
// endregion:   --- modules

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
impl BehaviorInstance for WasEntryUpdated {
	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		self.sequence_id = 0;
		if let Some(key) = behavior.remappings.find(&ENTRY.into()) {
			match strip_bb_pointer(&key) {
				Some(stripped) => self.entry_key = behavior.get::<String>(&stripped)?.into(),
				None => self.entry_key = key,
			}
			Ok(())
		} else {
			Err(BehaviorError::PortNotDeclared(
				"entry".into(),
				behavior.description().name().clone(),
			))
		}
	}

	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let sequence_id = behavior.get_sequence_id(&self.entry_key)?;
		if sequence_id == self.sequence_id {
			Ok(BehaviorState::Failure)
		} else {
			self.sequence_id = sequence_id;
			Ok(BehaviorState::Success)
		}
	}
}

impl BehaviorStatic for WasEntryUpdated {
	fn provided_ports() -> PortList {
		port_list![input_port!(
			String,
			ENTRY,
			"",
			"The blackboard entry to check."
		)]
	}
}
// endregion:   --- WasEntryUpdated
