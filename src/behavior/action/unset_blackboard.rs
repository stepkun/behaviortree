// Copyright © 2025 Stephan Kunz
//! [`UnsetBlackboard`] [`Action`] implementation.

// region:      --- modules
use crate::{
	BehaviorDescription, BehaviorExecution, BehaviorKind, BehaviorTreeFactory, EMPTY_STR,
	behavior::{Behavior, BehaviorCreationFn, BehaviorData, BehaviorResult, BehaviorState},
	input_port,
	port::PortList,
	port_list,
	tree::BehaviorTreeElementList,
};
use alloc::{
	boxed::Box,
	string::{String, ToString},
};
use core::{any::Any, fmt::Debug, marker::PhantomData, str::FromStr};
use tinyscript::SharedRuntime;
// endregion:   --- modules

// region:		--- globals
/// Port name literals
const KEY: &str = "key";
// endregion:	--- globals

// region:      --- UnsetBlackboard
/// The [`UnsetBlackboard`] behavior is used to delete a value of type T
/// from the Blackboard specified via port `key`.
/// Will return Success whether the entry exists or not.
///
/// The behavior is gated behind feature `unset_blackboard`.
#[derive(Default)]
pub struct UnsetBlackboard<T>
where
	T: Clone + Debug + Default + FromStr + ToString + Send + Sync + 'static,
{
	_marker: PhantomData<T>,
}

impl<T> BehaviorExecution for UnsetBlackboard<T>
where
	T: Clone + Debug + Default + FromStr + ToString + Send + Sync + 'static,
{
	fn as_any(&self) -> &dyn Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}

	fn creation_fn() -> Box<BehaviorCreationFn> {
		alloc::boxed::Box::new(|| alloc::boxed::Box::new(Self::default()))
	}

	fn kind() -> BehaviorKind {
		BehaviorKind::Action
	}

	fn static_provided_ports(&self) -> PortList {
		Self::provided_ports()
	}
}

#[async_trait::async_trait]
impl<T> Behavior for UnsetBlackboard<T>
where
	T: Clone + Debug + Default + FromStr + ToString + Send + Sync,
{
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let key = behavior.get::<String>(KEY)?;
		behavior.delete::<String>(&key)?;

		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(
			String,
			KEY,
			EMPTY_STR,
			"Key of the entry to remove"
		),]
	}
}

impl<T> UnsetBlackboard<T>
where
	T: Clone + Debug + Default + FromStr + ToString + Send + Sync,
{
	/// Creates a `creation_fn()` for `UnsetBlackboard` with the given state.
	#[must_use]
	pub fn create_fn() -> Box<BehaviorCreationFn> {
		Box::new(move || Box::new(Self::default()))
	}

	/// Registers the `UnsetBlackboard` behavior in the factory.
	/// # Errors
	/// - if registration fails
	pub fn register(
		factory: &mut BehaviorTreeFactory,
		name: &str,
		groot2: bool,
	) -> Result<(), crate::factory::error::Error> {
		let bhvr_desc = BehaviorDescription::new(name, name, BehaviorKind::Action, groot2, Self::provided_ports());
		let bhvr_creation_fn = Self::create_fn();
		factory
			.registry_mut()
			.add_behavior(bhvr_desc, bhvr_creation_fn)
	}
}
// endregion:   --- UnsetBlackboard
