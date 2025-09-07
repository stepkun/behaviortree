// Copyright © 2025 Stephan Kunz
//! [`SubTree`]  implementation.

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate::{
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorExecution, BehaviorKind, BehaviorResult, BehaviorState},
	port::PortList,
	tree::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:      --- SubTree
/// A `Subtree` is a placeholder for behavior (sub)trees with its own [`BehaviorKind`].
#[derive(Default)]
pub struct SubTree;

impl BehaviorExecution for SubTree {
	fn as_any(&self) -> &dyn core::any::Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
		self
	}

	fn creation_fn() -> Box<crate::behavior::BehaviorCreationFn> {
		alloc::boxed::Box::new(|| alloc::boxed::Box::new(Self))
	}

	fn kind() -> crate::prelude::BehaviorKind {
		BehaviorKind::SubTree
	}

	fn static_provided_ports(&self) -> PortList {
		PortList::default()
	}
}

#[async_trait::async_trait]
impl Behavior for SubTree {
	#[inline]
	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		if children.is_empty() || children.len() > 1 {
			return Err(BehaviorError::Composition("a subtree must have exactly 1 child".into()));
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
// endregion:   --- SubTree
