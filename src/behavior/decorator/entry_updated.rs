// Copyright © 2025 Stephan Kunz

//! `EntryUpdated` behavior implementation
//!

// region:      --- modules
use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use tinyscript::SharedRuntime;

use crate as behaviortree;
use crate::{
    ConstString, Decorator, ENTRY,
    behavior::{
        BehaviorData, BehaviorError, BehaviorInstance, BehaviorResult, BehaviorState,
        BehaviorStatic,
    },
    port::{PortList, strip_bb_pointer},
    tree::tree_element_list::ConstBehaviorTreeElementList,
};
use crate::{input_port, port_list};
// endregion:   --- modules

// region:      --- EntryUpdated
/// The `EntryUpdated` behavior checks the sequence number of a blackboard entry
/// to determine whether the entry was updated since last check (which will be true the first time).
/// - If it has been changed, the child will be executed and its state returned.
/// - Otherwise the value of `state_if_not` will be returned.
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
}

#[async_trait::async_trait]
impl BehaviorInstance for EntryUpdated {
    #[inline]
    fn on_halt(&mut self) -> Result<(), BehaviorError> {
        self.sequence_id = usize::MIN;
        self.is_running = false;
        Ok(())
    }

    fn on_start(
        &mut self,
        behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> Result<(), BehaviorError> {
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
        children: &mut ConstBehaviorTreeElementList,
        runtime: &SharedRuntime,
    ) -> BehaviorResult {
        if self.is_running {
            let state = children[0].tick(runtime).await?;
            self.is_running = state == BehaviorState::Running;
            return Ok(state);
        }

        let sequence_id = behavior.get_sequence_id(&self.entry_key)?;
        if sequence_id == self.sequence_id {
            Ok(self.state_if_not)
        } else {
            self.sequence_id = sequence_id;
            let state = children[0].tick(runtime).await?;
            self.is_running = state == BehaviorState::Running;
            return Ok(state);
        }
    }
}

impl BehaviorStatic for EntryUpdated {
    fn provided_ports() -> PortList {
        port_list![input_port!(
            String,
            ENTRY,
            "",
            "The blackboard entry to monitor."
        )]
    }
}
// endregion:   --- EntryUpdated
