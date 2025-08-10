// Copyright © 2025 Stephan Kunz

//! Built in `AlwaysXxx` behavior
//!

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate as behaviortree;
use crate::behavior::{BehaviorData, BehaviorError};
use crate::{
    Action,
    behavior::{BehaviorInstance, BehaviorResult, BehaviorState, BehaviorStatic},
    tree::ConstBehaviorTreeElementList,
};
//endregion:    --- modules

// region:		--- ChangeStateAfter
/// The `ChangeStateAfter` behavior returns
/// - the stored [`BehaviorState`] `state2` after the amount of ticks given by `count`,
/// - the [`BehaviorState`] `state1` just one tick before reaching `count`,
/// - before that the [`BehaviorState::Running`].
#[derive(Action, Debug, Default)]
pub struct ChangeStateAfter {
    /// The [`BehaviorState`] to return initially.
    state1: BehaviorState,
    /// The [`BehaviorState`] to return finally.
    state2: BehaviorState,
    /// The amount of ticks after which the state2 will be returned.
    count: u8,
    /// The remaining ticks until state2 will be returned.
    remaining: u8,
}

#[async_trait::async_trait]
impl BehaviorInstance for ChangeStateAfter {
    fn on_halt(&mut self) -> Result<(), BehaviorError> {
        self.remaining = self.count;
        Ok(())
    }

    fn on_start(
        &mut self,
        _behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> Result<(), BehaviorError> {
        self.remaining = self.count;
        Ok(())
    }

    async fn tick(
        &mut self,
        _behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> BehaviorResult {
        Ok(if self.remaining == 0 {
            self.state2
        } else if self.remaining == 1 {
            self.remaining -= 1;
            self.state1
        } else {
            self.remaining -= 1;
            BehaviorState::Running
        })
    }
}

impl BehaviorStatic for ChangeStateAfter {}

impl ChangeStateAfter {
    /// Constructor with arguments.
    #[must_use]
    pub const fn new(state1: BehaviorState, state2: BehaviorState, count: u8) -> Self {
        Self {
            state1,
            state2,
            count,
            remaining: count,
        }
    }

    /// Initialization function.
    pub const fn initialize(&mut self, state1: BehaviorState, state2: BehaviorState, count: u8) {
        self.state1 = state1;
        self.state2 = state2;
        self.count = count;
        self.remaining = count;
    }
}
// endregion:	--- ChangeStateAfter
