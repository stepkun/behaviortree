// Copyright © 2025 Stephan Kunz

//! Built in [`Delay`] decorator

// region:      --- modules
use alloc::boxed::Box;
use core::time::Duration;
use tinyscript::SharedRuntime;
#[cfg(feature = "std")]
use tokio::task::JoinHandle;

use crate::behavior::{BehaviorData, BehaviorError};
use crate::tree::ConstBehaviorTreeElementList;
use crate::{self as behaviortree, DELAY_MSEC};
use crate::{
    Behavior,
    behavior::{BehaviorInstance, BehaviorKind, BehaviorResult, BehaviorState, BehaviorStatic},
    input_port,
    port::PortList,
    port_list,
};
//endregion:    --- modules

// region:		--- Delay
/// The [`Delay`] decorator will introduce a delay given by the port `delay_msec` and then tick its child.
/// Consider also using the action [`Sleep`](crate::behavior::action::Sleep)
#[derive(Behavior, Debug, Default)]
pub struct Delay {
    handle: Option<JoinHandle<()>>,
}

#[async_trait::async_trait]
impl BehaviorInstance for Delay {
    #[inline]
    fn on_halt(&mut self) -> Result<(), BehaviorError> {
        self.handle = None;
        Ok(())
    }

    fn on_start(
        &mut self,
        behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> Result<(), BehaviorError> {
        let millis: u64 = behavior.get(DELAY_MSEC)?;
        self.handle = Some(tokio::task::spawn(async move {
            tokio::time::sleep(Duration::from_millis(millis)).await;
        }));
        behavior.set_state(BehaviorState::Running);
        Ok(())
    }

    async fn tick(
        &mut self,
        _behavior: &mut BehaviorData,
        children: &mut ConstBehaviorTreeElementList,
        runtime: &SharedRuntime,
    ) -> BehaviorResult {
        if let Some(handle) = self.handle.as_ref() {
            if handle.is_finished() {
                let state = children[0].tick(runtime).await?;
                if state.is_completed() {
                    children.halt(runtime)?;
                    Ok(BehaviorState::Success)
                } else {
                    Ok(state)
                }
            } else {
                Ok(BehaviorState::Running)
            }
        } else {
            Ok(BehaviorState::Failure)
        }
    }
}

impl BehaviorStatic for Delay {
    fn kind() -> BehaviorKind {
        BehaviorKind::Decorator
    }

    fn provided_ports() -> PortList {
        port_list![input_port!(
            u64,
            DELAY_MSEC,
            "",
            "Tick the child after a few milliseconds."
        )]
    }
}
// endregion:	--- Delay
