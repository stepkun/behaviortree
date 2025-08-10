// Copyright © 2025 Stephan Kunz

//! `Fallback` behavior implementation
//!

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate::behavior::BehaviorData;
use crate::{self as behaviortree, IDLE};
use crate::{
    Control,
    behavior::{
        BehaviorInstance, BehaviorResult, BehaviorState, BehaviorStatic, error::BehaviorError,
    },
    tree::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- Fallback
/// The `Fallback` behavior is used to try different strategies until one succeeds.
/// If any child returns RUNNING, previous children will NOT be ticked again.
/// - If all the children return FAILURE, this node returns FAILURE.
/// - If a child returns RUNNING, this node returns RUNNING.
/// - If a child returns SUCCESS, stop the loop and return SUCCESS.
#[derive(Control, Debug)]
pub struct Fallback {
    /// Defaults to '0'
    child_idx: usize,
    /// Defaults to 'true'
    all_skipped: bool,
}

impl Default for Fallback {
    fn default() -> Self {
        Self {
            child_idx: 0,
            all_skipped: true,
        }
    }
}
#[async_trait::async_trait]
impl BehaviorInstance for Fallback {
    #[inline]
    fn on_halt(&mut self) -> Result<(), BehaviorError> {
        self.child_idx = 0;
        self.all_skipped = true;
        Ok(())
    }

    #[inline]
    fn on_start(
        &mut self,
        behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> Result<(), BehaviorError> {
        behavior.set_state(BehaviorState::Running);
        Ok(())
    }

    async fn tick(
        &mut self,
        _behavior: &mut BehaviorData,
        children: &mut ConstBehaviorTreeElementList,
        runtime: &SharedRuntime,
    ) -> BehaviorResult {
        while self.child_idx < children.len() {
            let child = &mut children[self.child_idx];
            let new_state = child.tick(runtime).await?;

            self.all_skipped &= new_state == BehaviorState::Skipped;

            match new_state {
                BehaviorState::Failure | BehaviorState::Skipped => {
                    self.child_idx += 1;
                }
                BehaviorState::Idle => {
                    return Err(BehaviorError::State("Fallback".into(), IDLE.into()));
                }
                BehaviorState::Running => return Ok(BehaviorState::Running),
                BehaviorState::Success => {
                    children.halt(runtime)?;
                    self.child_idx = 0;
                    return Ok(BehaviorState::Success);
                }
            }
        }

        if self.child_idx >= children.len() {
            children.halt(runtime)?;
            self.child_idx = 0;
        }

        if self.all_skipped {
            Ok(BehaviorState::Skipped)
        } else {
            Ok(BehaviorState::Failure)
        }
    }
}

impl BehaviorStatic for Fallback {}
// endregion:   --- Fallback
