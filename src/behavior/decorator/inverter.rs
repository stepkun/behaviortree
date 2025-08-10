// Copyright © 2025 Stephan Kunz

//! `Inverter` behavior implementation
//!

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate as behaviortree;
use crate::behavior::{BehaviorData, IDLE};
use crate::{
    Decorator,
    behavior::{
        BehaviorInstance, BehaviorResult, BehaviorState, BehaviorStatic, error::BehaviorError,
    },
    tree::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- Inverter
/// The `Inverter` behavior is used invert the childs outcome:
/// - If child returns Success, this behavior returns Failure.
/// - If child returns Failure, this behavior returns Success.
/// - If child returns Skipped or Running, this state will be returned.
#[derive(Decorator, Debug, Default)]
pub struct Inverter;

#[async_trait::async_trait]
impl BehaviorInstance for Inverter {
    async fn tick(
        &mut self,
        _behavior: &mut BehaviorData,
        children: &mut ConstBehaviorTreeElementList,
        runtime: &SharedRuntime,
    ) -> BehaviorResult {
        let child = &mut children[0];
        let new_state = child.tick(runtime).await?;

        match new_state {
            BehaviorState::Failure => {
                children.halt(runtime)?;
                Ok(BehaviorState::Success)
            }
            BehaviorState::Idle => Err(BehaviorError::State("Inverter".into(), IDLE.into())),
            state @ (BehaviorState::Running | BehaviorState::Skipped) => Ok(state),
            BehaviorState::Success => {
                children.halt(runtime)?;
                Ok(BehaviorState::Failure)
            }
        }
    }
}

impl BehaviorStatic for Inverter {}
// endregion:   --- Inverter
