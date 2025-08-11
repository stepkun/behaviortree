// Copyright © 2025 Stephan Kunz

//! `SetBlackboard` behavior implementation
//!

// region:      --- modules
use alloc::string::String;
use alloc::{boxed::Box, string::ToString};
use core::marker::PhantomData;
use core::str::FromStr;
use tinyscript::SharedRuntime;

use crate::behavior::BehaviorData;
use crate::port::{PortList, strip_bb_pointer};
use crate::{self as behaviortree, OUTPUT_KEY, VALUE};
use crate::{
    Action,
    behavior::{BehaviorInstance, BehaviorResult, BehaviorState, BehaviorStatic},
    tree::ConstBehaviorTreeElementList,
};
use crate::{inout_port, input_port, port_list};
// endregion:   --- modules

// region:      --- SetBlackboard
/// The [`SetBlackboard`] behavior is used to store a value of type T
/// into an entry of the Blackboard specified via port `output_key`.
///
#[derive(Action, Default)]
pub struct SetBlackboard<T>
where
    T: Clone + Default + FromStr + ToString + Send + Sync + 'static,
{
    _marker: PhantomData<T>,
}

#[async_trait::async_trait]
impl<T> BehaviorInstance for SetBlackboard<T>
where
    T: Clone + Default + FromStr + ToString + Send + Sync,
{
    async fn tick(
        &mut self,
        behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> BehaviorResult {
        let value = behavior.get::<T>(VALUE)?;
        let key = behavior.get::<String>(OUTPUT_KEY)?;
        match strip_bb_pointer(&key) {
            Some(stripped_key) => {
                behavior.set(&stripped_key, value)?;
            }
            None => {
                behavior.set(&key, value)?;
            }
        }

        Ok(BehaviorState::Success)
    }
}

impl<T> BehaviorStatic for SetBlackboard<T>
where
    T: Clone + Default + FromStr + ToString + Send + Sync,
{
    fn provided_ports() -> PortList {
        port_list![
            input_port!(T, VALUE, "", "Value to be written into the output_key"),
            inout_port!(
                String,
                OUTPUT_KEY,
                "",
                "Name of the blackboard entry where the value should be written"
            ),
        ]
    }
}
// endregion:   --- SetBlackboard
