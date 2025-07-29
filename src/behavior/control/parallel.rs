// Copyright © 2025 Stephan Kunz

//! `Parallel` behavior implementation
//!

// region:      --- modules
use alloc::boxed::Box;
use alloc::collections::btree_set::BTreeSet;
use tinyscript::SharedRuntime;

use crate as behaviortree;
use crate::behavior::error::BehaviorError;
use crate::behavior::{BehaviorData, IDLE};
use crate::{
    Behavior,
    behavior::{BehaviorInstance, BehaviorKind, BehaviorResult, BehaviorState, BehaviorStatic},
    port::PortList,
    tree::ConstBehaviorTreeElementList,
};
use crate::{input_port, port_list};
// endregion:   --- modules

// region:      --- Parallel
/// A `Parallel` ticks executes children in
///
#[derive(Behavior, Debug)]
pub struct Parallel {
    /// The minimum needed Successes to retrun a Success.
    /// "-1" signals any number.
    success_threshold: i32,
    /// The maximum allowed failures.
    /// "-1" signals any number.
    failure_threshold: i32,
    /// The amount of completed sub behaviors that succeeded.
    success_count: i32,
    /// The amount of completed sub behaviors that failed.
    failure_count: i32,
    /// The list of completed sub behaviors
    completed_list: BTreeSet<usize>,
}

/// The port names
const SUCCESS_COUNT: &str = "success_count";
const FAILURE_COUNT: &str = "failure_count";

impl Default for Parallel {
    fn default() -> Self {
        Self {
            success_threshold: -1,
            failure_threshold: -1,
            success_count: 0,
            failure_count: 0,
            completed_list: BTreeSet::default(),
        }
    }
}

#[async_trait::async_trait]
impl BehaviorInstance for Parallel {
    fn on_halt(&mut self) -> Result<(), BehaviorError> {
        self.success_threshold = -1;
        self.failure_threshold = -1;
        self.completed_list.clear();
        self.success_count = 0;
        self.failure_count = 0;
        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::match_same_arms)]
    fn on_start(
        &mut self,
        behavior: &mut BehaviorData,
        children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> Result<(), BehaviorError> {
        // check composition only once at start
        self.success_threshold = behavior.get(SUCCESS_COUNT).unwrap_or(-1);
        self.failure_threshold = behavior.get(FAILURE_COUNT).unwrap_or(-1);

        let children_count = children.len();

        if (children_count as i32) < self.success_threshold {
            return Err(BehaviorError::Composition(
                "Number of children is less than the threshold. Can never succeed.".into(),
            ));
        }

        if (children_count as i32) < self.failure_threshold {
            return Err(BehaviorError::Composition(
                "Number of children is less than the threshold. Can never fail.".into(),
            ));
        }
        behavior.set_state(BehaviorState::Running);
        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::set_contains_or_insert)]
    async fn tick(
        &mut self,
        _behavior: &mut BehaviorData,
        children: &mut ConstBehaviorTreeElementList,
        runtime: &SharedRuntime,
    ) -> BehaviorResult {
        let children_count = children.len();

        let mut skipped_count = 0;

        for i in 0..children_count {
            // Skip completed node
            if !self.completed_list.contains(&i) {
                let child = &mut children[i];
                match child.tick(runtime).await? {
                    BehaviorState::Skipped => skipped_count += 1,
                    BehaviorState::Success => {
                        self.completed_list.insert(i);
                        self.success_count += 1;
                    }
                    BehaviorState::Failure => {
                        self.completed_list.insert(i);
                        self.failure_count += 1;
                    }
                    BehaviorState::Running => {}
                    // Throw error, should never happen
                    BehaviorState::Idle => {
                        return Err(BehaviorError::State("Parallel".into(), IDLE.into()));
                    }
                }
            }

            let sum = self.failure_count + self.success_count + skipped_count;
            if sum >= children_count as i32 {
                let state = if skipped_count == children_count as i32 {
                    BehaviorState::Skipped
                } else if self.failure_threshold <= 0 && self.success_threshold <= 0 {
                    BehaviorState::Success
                } else if self.failure_threshold <= 0 {
                    if self.success_count >= self.success_threshold {
                        BehaviorState::Success
                    } else {
                        BehaviorState::Failure
                    }
                } else if self.failure_count >= self.failure_threshold {
                    BehaviorState::Failure
                } else {
                    BehaviorState::Success
                };

                self.completed_list.clear();
                self.success_count = 0;
                self.failure_count = 0;
                children.halt(runtime)?;

                return Ok(state);
            }
        }

        Ok(BehaviorState::Running)
    }
}

impl BehaviorStatic for Parallel {
    fn kind() -> BehaviorKind {
        BehaviorKind::Control
    }

    fn provided_ports() -> PortList {
        port_list![
            input_port!(i32, SUCCESS_COUNT),
            input_port!(i32, FAILURE_COUNT)
        ]
    }
}
// endregion:   --- Parallel
