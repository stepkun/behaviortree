// Copyright © 2025 Stephan Kunz

//! `ForceFailure` behavior implementation
//!

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate as behaviortree;
use crate::behavior::{BehaviorData, IDLE};
use crate::{
    Behavior,
    behavior::{
        BehaviorInstance, BehaviorKind, BehaviorResult, BehaviorState, BehaviorStatic,
        error::BehaviorError,
    },
    tree::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- ForceState
/// The `ForceState` behavior is used to return a certain state, independant of what the child returned.
/// - If child returns Failure or Success, this behavior returns the stored [`BehaviorState`].
/// - If child returns any other state, that state will be returned.
#[derive(Behavior, Debug, Default)]
pub struct ForceState {
    state: BehaviorState,
}

#[async_trait::async_trait]
impl BehaviorInstance for ForceState {
    async fn tick(
        &mut self,
        _behavior: &mut BehaviorData,
        children: &mut ConstBehaviorTreeElementList,
        runtime: &SharedRuntime,
    ) -> BehaviorResult {
        let child = &mut children[0];
        let new_state = child.tick(runtime).await?;

        match new_state {
            BehaviorState::Failure | BehaviorState::Success => {
                children.halt(runtime)?;
                Ok(self.state)
            }
            BehaviorState::Idle => Err(BehaviorError::State("ForceState".into(), IDLE.into())),
            state @ (BehaviorState::Running | BehaviorState::Skipped) => Ok(state),
        }
    }
}

impl BehaviorStatic for ForceState {
    fn kind() -> BehaviorKind {
        BehaviorKind::Decorator
    }
}

impl ForceState {
    /// Constructor with arguments.
    #[must_use]
    pub const fn new(state: BehaviorState) -> Self {
        Self { state }
    }

    /// Initialization function.
    pub const fn initialize(&mut self, state: BehaviorState) {
        self.state = state;
    }
}
// endregion:   --- ForceState
