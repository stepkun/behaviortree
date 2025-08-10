// Copyright © 2025 Stephan Kunz

//! `SubTree` behavior implementation
//!

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate as behaviortree;
use crate::behavior::{BehaviorData, BehaviorError, BehaviorState};
use crate::{
    Decorator,
    behavior::{BehaviorInstance, BehaviorResult, BehaviorStatic},
    tree::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- SubTree
/// A `Subtree` is a placeholder for behavior (sub)trees with its own [`BehaviorKind`].
#[derive(Decorator, Debug, Default)]
pub struct SubTree;

#[async_trait::async_trait]
impl BehaviorInstance for SubTree {
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
        children[0].tick(runtime).await
    }
}

impl BehaviorStatic for SubTree {}
// endregion:   --- SubTree
