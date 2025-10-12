// Copyright © 2025 Stephan Kunz
//! [`EntryUpdated`] [`Decorator`] implementation.

// region:      --- modules
use crate::{
	self as behaviortree, ConstString, Decorator, EMPTY_STR,
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorResult, BehaviorState},
	input_port,
	port::PortList,
	port_list,
	tree::BehaviorTreeElementList,
};
use alloc::sync::Arc;
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

// region:      --- EntryUpdated
/// The `EntryUpdated` behavior checks the sequence number of a blackboard entry
/// to determine whether the entry was updated since last check (which will be true the first time).
/// - If it has been changed, the child will be executed and its state returned.
/// - Otherwise the value of `state_if_not` will be returned.
///
/// The behavior is gated behind feature `entry_updated`.
/// There are the predefined variants
/// - SkipUnlessUpdated: gated behind feature `skip_unless_updated`
/// - WaitValueUpdated: gated behind feature `wait_value_updated`
///
/// The raw version is gated behind feature `pop_from_queue`.
///
/// # Errors
/// If the entry does not exist
#[derive(Decorator, Debug, Default)]
pub struct EntryUpdated {
	/// ID of the last checked update
	/// The default of `usize::MIN` is used as never read
	sequence_id: usize,
	/// Still running the child
	is_running: bool,
	/// What to return if key is not updated
	state_if_not: BehaviorState,
	/// The entry to monitor
	entry_key: ConstString,
}

impl EntryUpdated {
	/// Create the behavior with a non default [`BehaviorState`] to return.
	/// The default state is [`BehaviorState::Idle`].
	#[must_use]
	pub fn new(state: BehaviorState) -> Self {
		Self {
			sequence_id: usize::MIN,
			is_running: false,
			state_if_not: state,
			entry_key: Arc::default(),
		}
	}

	/// Initialization function.
	pub const fn initialize(&mut self, state: BehaviorState) {
		self.state_if_not = state;
	}
}

#[async_trait::async_trait]
impl Behavior for EntryUpdated {
	#[inline]
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		self.sequence_id = usize::MIN;
		self.is_running = false;
		Ok(())
	}

	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		if let Some(key) = behavior.remappings().find(ENTRY) {
			self.entry_key = key;
			Ok(())
		} else {
			Err(BehaviorError::PortNotDeclared {
				port: "entry".into(),
				behavior: behavior.name().clone(),
			})
		}
	}

	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		if self.is_running {
			let state = children[0].tick(runtime).await?;
			self.is_running = state == BehaviorState::Running;
			return Ok(state);
		}

		let sequence_id = behavior.sequence_id(&self.entry_key)?;
		if sequence_id == self.sequence_id {
			Ok(self.state_if_not)
		} else {
			self.sequence_id = sequence_id;
			let state = children[0].tick(runtime).await?;
			self.is_running = state == BehaviorState::Running;
			return Ok(state);
		}
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(
			String,
			ENTRY,
			EMPTY_STR,
			"The blackboard entry to monitor."
		)]
	}
}
// endregion:   --- EntryUpdated
