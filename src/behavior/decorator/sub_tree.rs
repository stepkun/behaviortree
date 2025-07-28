// Copyright © 2025 Stephan Kunz

//! `SubTree` behavior implementation
//!

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate as behaviortree;
use crate::behavior::BehaviorData;
use crate::{
    Behavior,
    behavior::{BehaviorInstance, BehaviorKind, BehaviorResult, BehaviorStatic},
    tree::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- SubTree
/// A `Subtree` is a placeholder for behavior (sub)trees with its own [`BehaviorKind`].
#[derive(Behavior, Debug, Default)]
pub struct SubTree;

#[async_trait::async_trait]
impl BehaviorInstance for SubTree {
    async fn tick(
        &mut self,
        _behavior: &mut BehaviorData,
        children: &mut ConstBehaviorTreeElementList,
        runtime: &SharedRuntime,
    ) -> BehaviorResult {
        children[0].tick(runtime).await
    }
}

impl BehaviorStatic for SubTree {
    fn kind() -> BehaviorKind {
        BehaviorKind::SubTree
    }
}
// endregion:   --- SubTree
