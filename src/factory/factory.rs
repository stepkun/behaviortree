// Copyright © 2025 Stephan Kunz
//! Factory for creation and modification of [`BehaviorTree`]s.
//!
//! The factory ensures that a tree is properly created and libraries or plugins
//! are loaded properly and kept in memory as long as needed.

#[doc(hidden)]
#[cfg(feature = "std")]
extern crate std;

// region:      --- modules
use crate::{
	ConstString,
	behavior::{
		Behavior, BehaviorExecution, BehaviorKind, BehaviorState, ComplexBhvrTickFn, SimpleBehavior, SimpleBhvrTickFn,
		SubTree,
		action::PopFromQueue,
		action::{ChangeStateAfter, Script, SetBlackboard, Sleep, UnsetBlackboard},
		behavior_description::BehaviorDescription,
		condition::{ScriptCondition, WasEntryUpdated},
		control::{
			Fallback, IfThenElse, Parallel, ParallelAll, ReactiveFallback, ReactiveSequence, Sequence, SequenceWithMemory,
			Switch, WhileDoElse,
		},
		decorator::{
			Delay, EntryUpdated, ForceState, Inverter, KeepRunningUntilFailure, Loop, Precondition, Repeat,
			RetryUntilSuccessful, RunOnce, Timeout,
		},
	},
	port::PortList,
	tree::BehaviorTree,
	xml::parser::XmlParser,
};
#[cfg(feature = "std")]
use alloc::string::ToString;
use alloc::{boxed::Box, string::String, vec::Vec};
use databoard::Databoard;

use super::{error::Error, registry::BehaviorRegistry};
// endregion:   --- modules

// region:      --- BehaviorTreeFactory
/// Factory for creation and modification of [`BehaviorTree`]s
/// The default factory contains the elementary control behaviors:
/// - [`Fallback`]: the standard fallback control
/// - [`Sequence`]: the standard sequence control
/// - [`Parallel`]: the standard parallel contol with the ports
///   - `success_count`: the minimum of child successes to return Success
///   - `failure_count`: the maximum of child failures to return Success
///     (equivalent to the minimum of child failures to return Failure)
///
/// Note: Internally necessary are also
/// - [`SubTree`]: to enable sub trees including the root tree
pub struct BehaviorTreeFactory {
	registry: Box<BehaviorRegistry>,
}

impl Default for BehaviorTreeFactory {
	#[allow(clippy::expect_used)]
	fn default() -> Self {
		let mut f = Self {
			registry: Box::new(BehaviorRegistry::default()),
		};
		// minimum required behaviors for the factory to work
		// controls
		f.register_groot2_behavior_type::<Fallback>("Fallback")
			.expect("creating factory failed due to registration of [Fallback]");
		f.register_groot2_behavior_type::<Parallel>("Parallel")
			.expect("creating factory failed due to registration of [Parallel]");
		f.register_groot2_behavior_type::<Sequence>("Sequence")
			.expect("creating factory failed due to registration of [Sequence]");
		// subtree
		f.register_groot2_behavior_type::<SubTree>("SubTree")
			.expect("creating default factory failed due to registration of [SubTree]");

		f
	}
}

impl BehaviorTreeFactory {
	/// Access the registry.
	#[must_use]
	pub const fn registry(&self) -> &BehaviorRegistry {
		&self.registry
	}

	/// Access the registry mutable.
	#[must_use]
	pub const fn registry_mut(&mut self) -> &mut BehaviorRegistry {
		&mut self.registry
	}

	/// Create a factory with core set of behaviors which adds to the default behaviors:
	/// - Actions: [`Script`]
	/// - Conditions: [`ScriptCondition`], [`WasEntryUpdated`]
	/// - Controls: [`ParallelAll`], [`ReactiveFallback`], [`ReactiveSequence`], [`SequenceWithMemory`]
	/// - Decorators: [`Inverter`], [`Precondition`], [`RetryUntilSuccessful`],
	/// # Errors
	/// - if behaviors cannot be registered
	pub fn with_core_behaviors() -> Result<Self, Error> {
		let mut factory = Self::default();
		factory.register_core_behaviors()?;
		Ok(factory)
	}

	/// Create a factory with extended set of behaviors which adds to the core behaviors:
	/// - Actions: [`Sleep`]
	/// - Controls: [`IfThenElse`], [`WhileDoElse`]
	/// - Decorators: [`Delay`], [`KeepRunningUntilFailure`], [`Repeat`], [`RunOnce`], [`Timeout`],
	///   `SkipUnlessUpdated`, `WaitValueUpdated`
	/// # Errors
	/// - if behaviors cannot be registered
	pub fn with_extended_behaviors() -> Result<Self, Error> {
		let mut factory = Self::with_core_behaviors()?;
		factory.register_extended_behaviors()?;
		Ok(factory)
	}

	/// Create a factory with groot2 builtin behaviors which adds to the extended behaviors:
	/// - Actions: [`SetBlackboard`], [`UnsetBlackboard`]
	/// - Controls: `AsyncFallback`, `AsyncSequence`, `Switch2`, `Switch3`, `Switch4`, `Switch5`, `Switch6`,
	/// - Decorators: `LoopDouble`, `LoopString`
	///
	/// Note: It does not include the test behaviors `AlwaysFailure`, `AlwaysSuccess`, `ForceFailure` and `ForceSuccess`!
	///       These have to be registered separately with `factory.register_test_behaviors()`!
	/// # Errors
	/// - if behaviors cannot be registered
	pub fn with_groot2_behaviors() -> Result<Self, Error> {
		let mut factory = Self::with_extended_behaviors()?;
		factory.groot2_behaviors()?;
		Ok(factory)
	}

	/// Create a factory with all builtin behaviors which adds to the groot2 behaviors:
	/// - Actions: `PopBool`, `PopDouble`, `PopInt`, `PopString`
	/// - Decorators: `LoopBool`, `LoopInt`
	/// # Errors
	/// - if behaviors cannot be registered
	pub fn with_all_behaviors() -> Result<Self, Error> {
		let mut factory = Self::with_groot2_behaviors()?;
		factory.additional_behaviors()?;
		Ok(factory)
	}

	/// Register core behaviors:
	/// - Actions: [`Script`]
	/// - Conditions: [`ScriptCondition`], [`WasEntryUpdated`]
	/// - Controls: [`ParallelAll`], [`ReactiveFallback`], [`ReactiveSequence`], [`SequenceWithMemory`]
	/// - Decorators: [`Inverter`], [`Precondition`], [`RetryUntilSuccessful`],
	/// # Errors
	/// - if any registration fails
	pub fn register_core_behaviors(&mut self) -> Result<(), Error> {
		// actions
		self.register_groot2_behavior_type::<Script>("Script")?;

		// conditions
		self.register_groot2_behavior_type::<ScriptCondition>("ScriptCondition")?;
		self.register_groot2_behavior_type::<WasEntryUpdated>("WasEntryUpdated")?;

		// controls
		self.register_groot2_behavior_type::<ParallelAll>("ParallelAll")?;
		self.register_groot2_behavior_type::<ReactiveFallback>("ReactiveFallback")?;
		self.register_groot2_behavior_type::<ReactiveSequence>("ReactiveSequence")?;
		self.register_groot2_behavior_type::<SequenceWithMemory>("SequenceWithMemory")?;

		// decorators
		self.register_groot2_behavior_type::<Inverter>("Inverter")?;
		self.register_groot2_behavior_type::<Precondition>("Precondition")?;
		self.register_groot2_behavior_type::<RetryUntilSuccessful>("RetryUntilSuccessful")?;

		Ok(())
	}

	/// Register extended behaviors which includes:
	/// - Actions: [`Sleep`]
	/// - Controls: [`IfThenElse`], [`WhileDoElse`]
	/// - Decorators: [`Delay`], [`KeepRunningUntilFailure`], [`Repeat`], [`RunOnce`], [`Timeout`],
	///   `SkipUnlessUpdated`, `WaitValueUpdated`
	/// # Errors
	/// - if any registration fails
	pub fn register_extended_behaviors(&mut self) -> Result<(), Error> {
		// actions
		self.register_groot2_behavior_type::<Sleep>("Sleep")?;

		// conditions

		// controls
		self.register_groot2_behavior_type::<IfThenElse>("IfThenElse")?;
		self.register_groot2_behavior_type::<WhileDoElse>("WhileDoElse")?;

		// decorators
		self.register_groot2_behavior_type::<Delay>("Delay")?;
		self.register_groot2_behavior_type::<KeepRunningUntilFailure>("KeepRunningUntilFailure")?;
		self.register_groot2_behavior_type::<Repeat>("Repeat")?;
		self.register_groot2_behavior_type::<RunOnce>("RunOnce")?;
		self.register_groot2_behavior_type::<Timeout>("Timeout")?;

		let bhvr_desc = BehaviorDescription::new(
			"SkipUnlessUpdated",
			"SkipUnlessUpdated",
			EntryUpdated::kind(),
			true,
			EntryUpdated::provided_ports(),
		);
		let bhvr_creation_fn =
			Box::new(move || -> Box<dyn BehaviorExecution> { Box::new(EntryUpdated::new(BehaviorState::Skipped)) });
		self.registry_mut()
			.add_behavior(bhvr_desc, bhvr_creation_fn)?;

		let bhvr_desc = BehaviorDescription::new(
			"WaitValueUpdated",
			"WaitValueUpdated",
			EntryUpdated::kind(),
			true,
			EntryUpdated::provided_ports(),
		);
		let bhvr_creation_fn =
			Box::new(move || -> Box<dyn BehaviorExecution> { Box::new(EntryUpdated::new(BehaviorState::Running)) });
		self.registry_mut()
			.add_behavior(bhvr_desc, bhvr_creation_fn)?;

		Ok(())
	}

	/// Register additional groot2 builtin behaviors which includes:
	/// - Actions: [`SetBlackboard`], [`UnsetBlackboard`]
	/// - Controls: `AsyncFallback`, `AsyncSequence`, `Switch2`, `Switch3`, `Switch4`, `Switch5`, `Switch6`,
	/// - Decorators: `LoopDouble`, `LoopString`
	///
	/// Note: It does not include the test behaviors `AlwaysFailure`, `AlwaysSuccess`, `ForceFailure` and `ForceSuccess`!
	///       These have to be registered separately with `factory.register_test_behaviors()`!
	/// # Errors
	/// - if any registration fails
	pub fn groot2_behaviors(&mut self) -> Result<(), Error> {
		// actions
		self.register_groot2_behavior_type::<SetBlackboard<String>>("SetBlackboard")?;
		self.register_groot2_behavior_type::<UnsetBlackboard<String>>("UnsetBlackboard")?;

		// controls
		self.register_groot2_behavior_type::<Fallback>("AsyncFallback")?;
		self.register_groot2_behavior_type::<Sequence>("AsyncSequence")?;
		self.register_groot2_behavior_type::<Switch<2>>("Switch2")?;
		self.register_groot2_behavior_type::<Switch<3>>("Switch3")?;
		self.register_groot2_behavior_type::<Switch<4>>("Switch4")?;
		self.register_groot2_behavior_type::<Switch<5>>("Switch5")?;
		self.register_groot2_behavior_type::<Switch<6>>("Switch6")?;

		// decorators
		self.register_groot2_behavior_type::<Loop<f64>>("LoopDouble")?;
		self.register_groot2_behavior_type::<Loop<String>>("LoopString")?;

		Ok(())
	}

	/// Register additional builtin behaviors which includes:
	/// - Actions: `PopBool`, `PopDouble`, `PopInt`, `PopString`
	/// - Decorators: `LoopBool`, `LoopInt`
	/// # Errors
	/// - if any registration fails
	pub fn additional_behaviors(&mut self) -> Result<(), Error> {
		// actions
		self.register_behavior_type::<PopFromQueue<i32>>("PopInt")?;
		self.register_behavior_type::<PopFromQueue<bool>>("PopBool")?;
		self.register_behavior_type::<PopFromQueue<f64>>("PopDouble")?;
		self.register_behavior_type::<PopFromQueue<String>>("PopString")?;

		// decorators
		self.register_behavior_type::<Loop<bool>>("LoopBool")?;
		self.register_behavior_type::<Loop<i32>>("LoopInt")?;

		Ok(())
	}

	/// Register test behaviors which includes:
	/// - Actions: `AlwaysFailure`, `AlwaysRunning`, `AlwaysSuccess`,
	/// - Decorators: `ForceFailure`, `ForceRunning`, `ForceSuccess`
	/// # Errors
	/// - if any registration fails
	pub fn register_test_behaviors(&mut self) -> Result<(), Error> {
		// actions
		let bhvr_desc = BehaviorDescription::new(
			"AlwaysFailure",
			"AlwaysFailure",
			ChangeStateAfter::kind(),
			true,
			ChangeStateAfter::provided_ports(),
		);
		let bhvr_creation_fn = Box::new(move || -> Box<dyn BehaviorExecution> {
			Box::new(ChangeStateAfter::new(BehaviorState::Running, BehaviorState::Failure, 0))
		});
		self.registry_mut()
			.add_behavior(bhvr_desc, bhvr_creation_fn)?;

		let bhvr_desc = BehaviorDescription::new(
			"AlwaysRunning",
			"AlwaysRunning",
			ChangeStateAfter::kind(),
			false,
			ChangeStateAfter::provided_ports(),
		);
		let bhvr_creation_fn = Box::new(move || -> Box<dyn BehaviorExecution> {
			Box::new(ChangeStateAfter::new(BehaviorState::Running, BehaviorState::Running, 0))
		});
		self.registry_mut()
			.add_behavior(bhvr_desc, bhvr_creation_fn)?;

		let bhvr_desc = BehaviorDescription::new(
			"AlwaysSuccess",
			"AlwaysSuccess",
			ChangeStateAfter::kind(),
			true,
			ChangeStateAfter::provided_ports(),
		);
		let bhvr_creation_fn = Box::new(move || -> Box<dyn BehaviorExecution> {
			Box::new(ChangeStateAfter::new(BehaviorState::Running, BehaviorState::Success, 0))
		});
		self.registry_mut()
			.add_behavior(bhvr_desc, bhvr_creation_fn)?;

		// conditions

		// controls

		// decorators
		let bhvr_desc = BehaviorDescription::new(
			"ForceFailure",
			"ForceFailure",
			ForceState::kind(),
			true,
			ForceState::provided_ports(),
		);
		let bhvr_creation_fn =
			Box::new(move || -> Box<dyn BehaviorExecution> { Box::new(ForceState::new(BehaviorState::Failure)) });
		self.registry_mut()
			.add_behavior(bhvr_desc, bhvr_creation_fn)?;

		let bhvr_desc = BehaviorDescription::new(
			"ForceRunning",
			"ForceRunning",
			ForceState::kind(),
			false,
			ForceState::provided_ports(),
		);
		let bhvr_creation_fn =
			Box::new(move || -> Box<dyn BehaviorExecution> { Box::new(ForceState::new(BehaviorState::Running)) });
		self.registry_mut()
			.add_behavior(bhvr_desc, bhvr_creation_fn)?;

		let bhvr_desc = BehaviorDescription::new(
			"ForceSuccess",
			"ForceSuccess",
			ForceState::kind(),
			true,
			ForceState::provided_ports(),
		);
		let bhvr_creation_fn =
			Box::new(move || -> Box<dyn BehaviorExecution> { Box::new(ForceState::new(BehaviorState::Success)) });
		self.registry_mut()
			.add_behavior(bhvr_desc, bhvr_creation_fn)?;

		Ok(())
	}

	/// Register an enums key/value pair.
	/// # Errors
	/// - if the key is already used
	pub fn register_enum_tuple(&mut self, key: &str, value: i8) -> Result<(), Error> {
		self.registry.register_enum_tuple(key, value)
	}

	/// Clear previously registered behavior trees.
	pub fn clear_registered_behavior_trees(&mut self) {
		self.registry.clear_registered_trees();
	}

	/// Create a [`BehaviorTree`] directly from XML.
	/// # Errors
	/// - if XML is not well formatted
	/// - if no main tree is defined
	/// - if behaviors or subtrees are missing
	pub fn create_from_text(&mut self, xml: &str) -> Result<BehaviorTree, Error> {
		self.register_behavior_tree_from_text(xml)?;
		self.create_main_tree()
	}

	/// Create a [`BehaviorTree`] from previous registration.
	/// # Errors
	/// - if no main tree has been defined during regisration
	/// - if behaviors or subtrees are missing
	pub fn create_main_tree(&mut self) -> Result<BehaviorTree, Error> {
		if let Some(name) = self.registry.main_tree_id() {
			if name.is_empty() {
				self.create_tree("MainTree")
			} else {
				self.create_tree(&name)
			}
		} else {
			self.create_tree("MainTree")
		}
	}

	/// Create the named [`BehaviorTree`] from registration.
	/// # Errors
	/// - if no tree with `name` can be found
	/// - if behaviors or subtrees are missing
	pub fn create_tree(&mut self, name: &str) -> Result<BehaviorTree, Error> {
		let mut parser = XmlParser::default();
		match parser.create_tree_from_definition(name, &mut self.registry, None) {
			Ok(root) => Ok(BehaviorTree::new(root, &self.registry)),
			#[cfg(feature = "std")]
			Err(err) => Err(Error::Create(name.into(), err.to_string().into())),
			#[cfg(not(feature = "std"))]
			Err(_) => Err(Error::Create(name.into())),
		}
	}

	/// Create the named [`BehaviorTree`] from registration using external created blackboard.
	/// # Errors
	/// - if no tree with `name` can be found
	/// - if behaviors or subtrees are missing
	pub fn create_tree_with(&mut self, name: &str, blackboard: Databoard) -> Result<BehaviorTree, Error> {
		let mut parser = XmlParser::default();
		match parser.create_tree_from_definition(name, &mut self.registry, Some(blackboard)) {
			Ok(root) => Ok(BehaviorTree::new(root, &self.registry)),
			#[cfg(feature = "std")]
			Err(err) => Err(Error::Create(name.into(), err.to_string().into())),
			#[cfg(not(feature = "std"))]
			Err(_) => Err(Error::Create(name.into())),
		}
	}

	/// Prints out the list of registered behaviors.
	pub fn list_behaviors(&self) {
		self.registry.list_behaviors();
	}

	/// Register the behavior (sub)trees described by the XML.
	/// # Errors
	/// - on incorrect XML
	/// - if tree description is not in BTCPP v4
	/// - if tree is already registered
	pub fn register_behavior_tree_from_text(&mut self, xml: impl Into<ConstString>) -> Result<(), Error> {
		#[cfg(feature = "std")]
		{
			let dir = std::env::current_dir()?.to_string_lossy().into();
			match XmlParser::register_document(&mut self.registry, &xml.into(), dir) {
				Ok(()) => Ok(()),
				Err(err) => Err(Error::RegisterXml(err.to_string().into())),
			}
		}
		#[cfg(not(feature = "std"))]
		{
			match XmlParser::register_document(&mut self.registry, &xml.into()) {
				Ok(()) => Ok(()),
				Err(_) => Err(Error::RegisterXml),
			}
		}
	}

	/// Register the behavior (sub)trees described by the XML in the file.
	/// # Errors
	/// - on incorrect XML
	/// - if the given file path is not a valid path
	/// - if description is not 'BTCPP v4'
	/// - if a behavior is already registered
	/// - if a (sub)tree is already registered
	#[cfg(feature = "std")]
	pub fn register_behavior_tree_from_file(&mut self, file: impl Into<std::path::PathBuf>) -> Result<(), Error> {
		let file_path: std::path::PathBuf = file.into();
		if let Some(file_dir) = file_path.parent() {
			let dir: ConstString = if file_path.is_relative() {
				let mut dir = std::env::current_dir()?;
				dir.push(file_dir);
				dir.to_string_lossy().into()
			} else {
				file_dir.to_string_lossy().into()
			};
			let xml: ConstString = std::fs::read_to_string(file_path)?.into();
			//XmlParser::register_document(&mut self.registry, &xml, dir)
			match XmlParser::register_document(&mut self.registry, &xml, dir) {
				Ok(()) => Ok(()),
				Err(err) => Err(Error::RegisterXml(err.to_string().into())),
			}
		} else {
			Err(Error::RegisterXml("filepath without parent".into()))
		}
	}

	/// Get the name list of registered behavior trees.
	#[must_use]
	pub fn registered_behavior_trees(&self) -> Vec<ConstString> {
		self.registry.registered_behavior_trees()
	}

	/// Register a behavior plugin.
	/// For now it is  recommended, that
	/// - the plugin resides in the executables directory and
	/// - is compiled with the same `Rust` version.
	/// # Errors
	/// - if library is not found
	/// - if library does not provide the `extern "Rust" register(&mut BehaviorTreeFactory) -> i32` function
	/// # Panics
	/// - on OS other than `Windows` and `Linux`,
	/// - should not panic on supported OS unless some weird constellation is happening.
	#[cfg(feature = "std")]
	#[allow(unsafe_code)]
	pub fn register_from_plugin(&mut self, name: &str) -> Result<(), Error> {
		// create path from exe path
		// in dev environment maybe we have to remove a '/deps'
		if let Some(path) = std::env::current_exe()?.parent() {
			if let Some(str_path) = path.to_str() {
				let path = str_path.trim_end_matches("/deps").to_string();

				#[cfg(not(any(target_os = "linux", target_os = "windows")))]
				todo!("This plattform is not upported!");
				#[cfg(target_os = "linux")]
				let libname = path + "/lib" + name + ".so";
				#[cfg(target_os = "windows")]
				let libname = path + "\\" + name + ".dll";

				let lib = unsafe {
					let lib = libloading::Library::new(libname)?;
					let registration_fn: libloading::Symbol<unsafe extern "Rust" fn(&mut Self) -> u32> =
						lib.get(b"register")?;
					let res = registration_fn(&mut *self);
					if res != 0 {
						return Err(Error::RegisterLib(name.into(), res));
					}
					lib
				};

				// The Library must be kept in storage until the [`BehaviorTree`] is destroyed.
				// Therefore the library is handed over to the behavior registry and later referenced by any tree.
				self.registry.add_library(lib);
				Ok(())
			} else {
				Err(Error::InvalidPath { path: name.into() })
			}
		} else {
			Err(Error::InvalidPath { path: name.into() })
		}
	}

	/// Register a `Behavior` of type `<T>`.
	/// # Errors
	/// - if a behavior with that `name` is already registered
	pub fn register_behavior_type<T>(&mut self, name: &str) -> Result<(), Error>
	where
		T: BehaviorExecution,
	{
		let bhvr_desc = BehaviorDescription::new(name, name, T::kind(), false, T::provided_ports());
		let bhvr_creation_fn = T::creation_fn();
		self.registry
			.add_behavior(bhvr_desc, bhvr_creation_fn)
	}

	/// Register a `Behavior` of type `<T>` which is also builtin in Groot2.
	/// # Errors
	/// - if a behavior with that `name` is already registered
	fn register_groot2_behavior_type<T>(&mut self, name: &str) -> Result<(), Error>
	where
		T: BehaviorExecution,
	{
		let bhvr_desc = BehaviorDescription::new(name, name, T::kind(), true, T::provided_ports());
		let bhvr_creation_fn = T::creation_fn();
		self.registry
			.add_behavior(bhvr_desc, bhvr_creation_fn)
	}

	/// Register a function either as [`BehaviorKind::Action`] or as [`BehaviorKind::Condition`].
	/// # Errors
	/// - if a behavior with that `name` is already registered
	pub fn register_simple_function(
		&mut self,
		name: &str,
		tick_fn: SimpleBhvrTickFn,
		kind: BehaviorKind,
	) -> Result<(), Error> {
		let bhvr_desc = BehaviorDescription::new(name, name, kind, false, PortList::default());
		let bhvr_creation_fn = SimpleBehavior::create(tick_fn);
		self.registry
			.add_behavior(bhvr_desc, bhvr_creation_fn)
	}

	/// Register a function as [`BehaviorKind::Action`] or [`BehaviorKind::Condition`] which is using ports.
	/// # Errors
	/// - if a behavior with that `name` is already registered
	pub fn register_simple_function_with_ports(
		&mut self,
		name: &str,
		tick_fn: ComplexBhvrTickFn,
		kind: BehaviorKind,
		port_list: PortList,
	) -> Result<(), Error> {
		let bhvr_desc = BehaviorDescription::new(name, name, kind, false, port_list.clone());
		let bhvr_creation_fn = SimpleBehavior::new_create_with_ports(tick_fn, port_list);
		self.registry
			.add_behavior(bhvr_desc, bhvr_creation_fn)
	}
}
// endregion:   --- BehaviorTreeFactory
