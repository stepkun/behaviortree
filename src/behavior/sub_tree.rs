// Copyright © 2025 Stephan Kunz
//! [`SubTree`]  implementation.

// region:      --- modules
use crate::{
	BehaviorDescription, BehaviorTreeFactory,
	behavior::{
		Behavior, BehaviorData, BehaviorExecution, BehaviorKind, BehaviorResult, BehaviorState,
		error::Error as BehaviorError,
	},
	port::PortList,
	tree::BehaviorTreeElementList,
};
use alloc::boxed::Box;
use tinyscript::SharedRuntime;
// endregion:   --- modules

// region:      --- SubTree
/// A `Subtree` is a placeholder for behavior (sub)trees with its own [`BehaviorKind`].
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
		Self::provided_ports()
	}
}

#[async_trait::async_trait]
impl Behavior for SubTree {
	#[inline]
	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		if children.is_empty() || children.len() > 1 {
			return Err(BehaviorError::Composition {
				txt: "a subtree must have exactly 1 child".into(),
			});
		}
		behavior.set_state(BehaviorState::Running);
		Ok(())
	}

	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		children[0].tick(runtime).await
	}
}

impl SubTree {
	/// Creates a `creation_fn()` for a `SubTree` with the given configuration.
	fn create_fn() -> Box<crate::behavior::BehaviorCreationFn> {
		alloc::boxed::Box::new(|| alloc::boxed::Box::new(Self))
	}

	/// Registers the `SubTree` behavior in the factory.
	/// # Errors
	/// - if registration fails
	pub fn register(factory: &mut BehaviorTreeFactory, name: &str) -> Result<(), crate::factory::error::Error> {
		let bhvr_desc = BehaviorDescription::new(name, name, BehaviorKind::SubTree, true, Self::provided_ports());
		let bhvr_creation_fn = Self::create_fn();
		factory
			.registry_mut()
			.add_behavior(bhvr_desc, bhvr_creation_fn)
	}
}
// endregion:   --- SubTree
