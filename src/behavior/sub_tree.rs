// Copyright © 2025 Stephan Kunz

//! `SubTree` behavior implementation
//!

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate::{
	behavior::{
		Behavior, BehaviorData, BehaviorError, BehaviorExecution, BehaviorInstance, BehaviorKind, BehaviorResult,
		BehaviorState, BehaviorStatic,
	},
	port::PortList,
	tree::tree_element_list::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- SubTree
/// A `Subtree` is a placeholder for behavior (sub)trees with its own [`BehaviorKind`].
#[derive(Default)]
pub struct SubTree;

impl Behavior for SubTree {
	fn creation_fn() -> Box<crate::behavior::BehaviorCreationFn> {
		alloc::boxed::Box::new(|| alloc::boxed::Box::new(Self))
	}

	fn kind() -> crate::prelude::BehaviorKind {
		BehaviorKind::SubTree
	}
}

impl BehaviorExecution for SubTree {
	fn as_any(&self) -> &dyn core::any::Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
		self
	}

	fn static_provided_ports(&self) -> PortList {
		PortList::default()
	}
}

#[async_trait::async_trait]
impl BehaviorInstance for SubTree {
	#[inline]
	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		if children.is_empty() {
			return Err(BehaviorError::Composition("subtree must have 1 child".into()));
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
		children[0].tick(runtime).await
	}
}

impl BehaviorStatic for SubTree {}
// endregion:   --- SubTree
