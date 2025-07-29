// Copyright © 2025 Stephan Kunz

//! `Switch` behavior implementation
//!

// region:      --- modules
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use tinyscript::SharedRuntime;

use crate::behavior::{BehaviorData, IDLE};
use crate::input_port;
use crate::port::PortList;
use crate::{self as behaviortree, CASE, VARIABLE};
use crate::{
    Behavior,
    behavior::{
        BehaviorInstance, BehaviorKind, BehaviorResult, BehaviorState, BehaviorStatic,
        error::BehaviorError,
    },
    tree::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- Switch
/// The `Switch` behavior is .
#[derive(Behavior, Debug)]
pub struct Switch<const T: u8> {
    /// Defaults to T
    cases: u8,
    /// Defaults to '-1'
    running_child_index: i32,
}

impl<const T: u8> Default for Switch<T> {
    fn default() -> Self {
        Self {
            cases: T,
            running_child_index: -1,
        }
    }
}

#[async_trait::async_trait]
impl<const T: u8> BehaviorInstance for Switch<T> {
    fn on_halt(&mut self) -> Result<(), BehaviorError> {
        self.cases = T;
        self.running_child_index = -1;
        Ok(())
    }

    fn on_start(
        &mut self,
        behavior: &mut BehaviorData,
        children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> Result<(), BehaviorError> {
        self.running_child_index = -1;

        // check composition
        if children.len() != (self.cases + 1) as usize {
            return Err(BehaviorError::Composition(
                "Wrong number of children in Switch behavior: must be (num_cases + 1)!".into(),
            ));
        }
        behavior.set_state(BehaviorState::Running);
        Ok(())
    }

    async fn tick(
        &mut self,
        behavior: &mut BehaviorData,
        children: &mut ConstBehaviorTreeElementList,
        runtime: &SharedRuntime,
    ) -> BehaviorResult {
        // default match index
        let default_index = i32::from(T);
        let mut match_index = i32::from(T);
        if let Ok(var) = behavior.get::<String>(VARIABLE) {
            for i in 0..T {
                let key = String::from(CASE) + &i.to_string();
                let x = behavior.get::<String>(&key)?;
                // @TODO: extend with enums, scripting, etc.
                if var == x {
                    match_index = i32::from(i);
                    break;
                }
            }
        }

        // stop child, if it is not the one that should run
        if self.running_child_index > 0
            && match_index != self.running_child_index
            && match_index <= default_index
        {
            #[allow(clippy::cast_sign_loss)]
            children[self.running_child_index as usize].halt_children(runtime)?;
        }

        #[allow(clippy::cast_sign_loss)]
        let state = children[match_index as usize].tick(runtime).await?;

        if state == BehaviorState::Skipped {
            // if the matching child is Skipped, should default be executed or
            // return just Skipped? Going with the latter for now.
            self.running_child_index = -1;
        } else if state == BehaviorState::Idle {
            return Err(BehaviorError::State("Switch".into(), IDLE.into()));
        } else if state == BehaviorState::Running {
            self.running_child_index = match_index;
        } else {
            children.halt(runtime)?;
            self.running_child_index = -1;
        }
        Ok(state)
    }
}

impl<const T: u8> BehaviorStatic for Switch<T> {
    fn kind() -> BehaviorKind {
        BehaviorKind::Control
    }

    fn provided_ports() -> PortList {
        let mut ports = PortList::default();
        let port = input_port!(String, VARIABLE);
        ports
            .add(port)
            .expect("providing port [variable] failed in behavior [Switch<T>]");

        for i in 0..T {
            let name = String::from(CASE) + &i.to_string();
            let port = input_port!(String, name.as_str());
            ports
                .add(port)
                .expect("providing port [case_T] failed in behavior [Switch<T>]");
        }
        ports
    }
}
// endregion:   --- Fallback
