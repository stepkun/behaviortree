// Copyright © 2025 Stephan Kunz
//! [`SetBlackboard`] [`Action`] implementation.

// region:      --- modules
use crate::{
	self as behaviortree, Action, BehaviorDescription, BehaviorKind, BehaviorTreeFactory, EMPTY_STR,
	behavior::{Behavior, BehaviorCreationFn, BehaviorData, BehaviorResult, BehaviorState},
	inout_port, input_port,
	port::PortList,
	port_list,
	tree::BehaviorTreeElementList,
};
use alloc::{boxed::Box, string::String, string::ToString};
use core::{fmt::Debug, marker::PhantomData, str::FromStr};
use databoard::check_board_pointer;
use tinyscript::SharedRuntime;
// endregion:   --- modules

// region:		--- globals
/// Port name literals
const OUTPUT_KEY: &str = "output_key";
const VALUE: &str = "value";
// endregion:	--- globals

// region:      --- SetBlackboard
/// The [`SetBlackboard`] behavior is used to store a value of type T
/// into an entry of the Blackboard specified via port `output_key`.
///
/// The behavior is gated behind feature `set_blackboard`.
#[derive(Action, Debug, Default)]
#[behavior(no_create, no_register, no_register_with)]
pub struct SetBlackboard<T>
where
	T: Clone + Debug + Default + FromStr + ToString + Send + Sync + 'static,
{
	_marker: PhantomData<T>,
}

#[async_trait::async_trait]
impl<T> Behavior for SetBlackboard<T>
where
	T: Clone + Debug + Default + FromStr + ToString + Send + Sync,
{
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let value = behavior.get::<T>(VALUE)?;
		let key = behavior.get::<String>(OUTPUT_KEY)?;
		match check_board_pointer(&key) {
			Ok(stripped_key) => behavior.set(stripped_key, value)?,
			Err(original_key) => behavior.set(original_key, value)?,
		};

		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list![
			input_port!(T, VALUE, EMPTY_STR, "Value to be written into the output_key"),
			inout_port!(
				String,
				OUTPUT_KEY,
				EMPTY_STR,
				"Name of the blackboard entry where the value should be written"
			),
		]
	}
}

impl<T> SetBlackboard<T>
where
	T: Clone + Debug + Default + FromStr + ToString + Send + Sync,
{
	/// Creates a `creation_fn()` for `SetBlackboard`.
	#[must_use]
	pub fn create_fn() -> Box<BehaviorCreationFn> {
		Box::new(move || Box::new(Self { _marker: PhantomData }))
	}

	/// Registers the `SetBlackboard` behavior in the factory.
	/// # Errors
	/// - if registration fails
	pub fn register_with(
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
// endregion:   --- SetBlackboard
