// Copyright © 2025 Stephan Kunz

//! Built in [`Timeout`] decorator

// region:      --- modules
use alloc::boxed::Box;
use core::time::Duration;
use tinyscript::SharedRuntime;
#[cfg(feature = "std")]
use tokio::task::JoinHandle;

use crate::{self as behaviortree, MSEC};
use crate::behavior::{BehaviorData, BehaviorError};
use crate::tree::ConstBehaviorTreeElementList;
use crate::{
    Behavior,
    behavior::{BehaviorInstance, BehaviorKind, BehaviorResult, BehaviorState, BehaviorStatic},
    input_port,
    port::PortList,
    port_list,
};
//endregion:    --- modules

// region:		--- Timeout
/// The [`Timeout`] decorator will halt its child after a period given by the port `msec`.
#[derive(Behavior, Debug, Default)]
pub struct Timeout {
    handle: Option<JoinHandle<()>>,
}

#[async_trait::async_trait]
impl BehaviorInstance for Timeout {
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
        let millis: u64 = behavior.get(MSEC)?;
        self.handle = Some(tokio::task::spawn(async move {
            tokio::time::sleep(Duration::from_millis(millis)).await;
        }));

        Ok(())
    }

    async fn tick(
        &mut self,
        _behavior: &mut BehaviorData,
        children: &mut ConstBehaviorTreeElementList,
        runtime: &SharedRuntime,
    ) -> BehaviorResult {
        if let Some(handle) = self.handle.as_ref() {
            let state = children[0].tick(runtime).await?;
            if state.is_completed() {
                children.halt(runtime)?;
                Ok(state)
            } else if handle.is_finished() {
                self.handle = None;
                children[0].halt_children(runtime)?;
                Ok(BehaviorState::Failure)
            } else {
                Ok(BehaviorState::Running)
            }
        } else {
            Ok(BehaviorState::Failure)
        }
    }
}

impl BehaviorStatic for Timeout {
    fn kind() -> BehaviorKind {
        BehaviorKind::Decorator
    }

    fn provided_ports() -> PortList {
        port_list![input_port!(
            u64,
            MSEC,
            "",
            "Timeout the child after a few milliseconds."
        )]
    }
}
// endregion:	--- Delay
