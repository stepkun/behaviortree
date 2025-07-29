// Copyright © 2025 Stephan Kunz

//! `Repeat` behavior implementation
//!

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate::{self as behaviortree, NUM_CYCLES};
use crate::behavior::{BehaviorData, IDLE};
use crate::{
    Behavior,
    behavior::{
        BehaviorInstance, BehaviorKind, BehaviorResult, BehaviorState, BehaviorStatic,
        error::BehaviorError,
    },
    input_port,
    port::PortList,
    port_list,
    tree::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- Repeat
/// The [`Repeat`] decorator is used to execute a child several times as long as it succeeds.
///
/// Example:
///
/// ```xml
/// <Repeat num_cycles="3">
///     <WaveHand/>
/// </Repeat>
/// ```
#[derive(Behavior, Debug)]
pub struct Repeat {
    /// Defaults to `-1`
    num_cycles: i32,
    /// Defaults to `0`
    repeat_count: i32,
}

impl Default for Repeat {
    fn default() -> Self {
        Self {
            num_cycles: -1,
            repeat_count: 0,
        }
    }
}

#[async_trait::async_trait]
impl BehaviorInstance for Repeat {
    fn on_halt(&mut self) -> Result<(), BehaviorError> {
        self.repeat_count = 0;
        Ok(())
    }

    fn on_start(
        &mut self,
        behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> Result<(), BehaviorError> {
        // Load num_cycles from the port value
        self.num_cycles = behavior.get::<i32>(NUM_CYCLES)?;

        Ok(())
    }

    async fn tick(
        &mut self,
        _behavior: &mut BehaviorData,
        children: &mut ConstBehaviorTreeElementList,
        runtime: &SharedRuntime,
    ) -> BehaviorResult {
        if self.repeat_count < self.num_cycles || self.num_cycles == -1 {
            let child = &mut children[0];
            let new_state = child.tick(runtime).await?;

            match new_state {
                BehaviorState::Failure => {
                    self.repeat_count = 0;
                    children.halt(runtime)?;
                    Ok(BehaviorState::Failure)
                }
                BehaviorState::Idle => Err(BehaviorError::State("Repeat".into(), IDLE.into())),
                BehaviorState::Running => return Ok(BehaviorState::Running),
                BehaviorState::Skipped => {
                    children.halt(runtime)?;
                    Ok(BehaviorState::Skipped)
                }
                BehaviorState::Success => {
                    self.repeat_count += 1;
                    children.halt(runtime)?;
                    Ok(BehaviorState::Running)
                }
            }
        } else {
            Ok(BehaviorState::Success)
        }
    }
}

impl BehaviorStatic for Repeat {
    fn kind() -> BehaviorKind {
        BehaviorKind::Decorator
    }

    fn provided_ports() -> PortList {
        port_list![input_port!(
            i32,
            NUM_CYCLES,
            "",
            "Repeat a successful child up to N times. Use -1 to create an infinite loop."
        )]
    }
}
// endregion:   --- RetryUntilSuccessful
