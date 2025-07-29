// Copyright © 2025 Stephan Kunz

//! Built in [`Sleep`] action behavior

// region:      --- modules
use alloc::boxed::Box;
use core::time::Duration;
use tinyscript::SharedRuntime;
#[cfg(feature = "std")]
use tokio::task::JoinHandle;

use crate::behavior::{BehaviorData, BehaviorError};
use crate::tree::ConstBehaviorTreeElementList;
use crate::{self as behaviortree, MSEC};
use crate::{
    Behavior,
    behavior::{BehaviorInstance, BehaviorKind, BehaviorResult, BehaviorState, BehaviorStatic},
    input_port,
    port::PortList,
    port_list,
};
//endregion:    --- modules

// region:		--- Sleep
/// The [`Sleep`] behavior sleeps for the amount of time given via port msec.
/// Consider also using the decorator [`Delay`](crate::behavior::decorator::Delay)
#[derive(Behavior, Debug, Default)]
pub struct Sleep {
    handle: Option<JoinHandle<()>>,
}

#[async_trait::async_trait]
impl BehaviorInstance for Sleep {
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
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> BehaviorResult {
        if let Some(handle) = self.handle.as_ref() {
            if handle.is_finished() {
                self.handle = None;
                Ok(BehaviorState::Success)
            } else {
                Ok(BehaviorState::Running)
            }
        } else {
            Ok(BehaviorState::Failure)
        }
    }
}

impl BehaviorStatic for Sleep {
    fn kind() -> BehaviorKind {
        BehaviorKind::Action
    }

    fn provided_ports() -> PortList {
        port_list![input_port!(u64, MSEC, "", "Time to sleep in [msec].")]
    }
}
// endregion:	--- Sleep
