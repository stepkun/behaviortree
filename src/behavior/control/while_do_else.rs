// Copyright © 2025 Stephan Kunz

//! `WhileDoElse` behavior implementation
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

// region:      --- WhileDoElse
/// The `WhileDoElse` behavior .
#[derive(Behavior, Debug, Default)]
pub struct WhileDoElse;

#[async_trait::async_trait]
impl BehaviorInstance for WhileDoElse {
    fn on_start(
        &mut self,
        behavior: &mut BehaviorData,
        children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> Result<(), BehaviorError> {
        // check composition only once at start
        if !(2..=3).contains(&children.len()) {
            return Err(BehaviorError::Composition(
                "WhileDoElse must have either 2 or 3 children.".into(),
            ));
        }
        behavior.set_state(BehaviorState::Running);
        Ok(())
    }

    async fn tick(
        &mut self,
        _behavior: &mut BehaviorData,
        children: &mut ConstBehaviorTreeElementList,
        runtime: &SharedRuntime,
    ) -> BehaviorResult {
        let children_count = children.len();

        let condition_status = children[0].tick(runtime).await?;

        if matches!(condition_status, BehaviorState::Running) {
            return Ok(BehaviorState::Running);
        }

        let status = match condition_status {
            BehaviorState::Failure => match children_count {
                3 => {
                    children.halt_at(1, runtime)?;
                    children[2].tick(runtime).await?
                }
                2 => BehaviorState::Failure,
                _ => {
                    return Err(BehaviorError::Composition(
                        "Should not happen in 'WhileDoElse'".into(),
                    ));
                }
            },
            BehaviorState::Idle => {
                return Err(BehaviorError::State("WhileDoElse".into(), IDLE.into()));
            }
            BehaviorState::Running => {
                return Ok(BehaviorState::Running);
            }
            BehaviorState::Skipped => {
                return Ok(BehaviorState::Skipped);
            }
            BehaviorState::Success => {
                if children_count == 3 {
                    children.halt_at(2, runtime)?;
                }

                children[1].tick(runtime).await?
            }
        };

        match status {
            BehaviorState::Running => Ok(BehaviorState::Running),
            status => {
                children.halt(runtime)?;
                Ok(status)
            }
        }
    }
}

impl BehaviorStatic for WhileDoElse {
    fn kind() -> BehaviorKind {
        BehaviorKind::Control
    }
}
// endregion:   --- WhileDoElse
