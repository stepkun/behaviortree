// Copyright © 2025 Stephan Kunz
//! Factory for creation and modification of [`BehaviorTree`]s.
//!
//! The factory ensures that a tree is properly created and libraries or plugins
//! are loaded properly and kept in memory as long as needed.

#[doc(hidden)]
#[cfg(feature = "std")]
extern crate std;

// region:      --- modules
use super::{error::Error, registry::BehaviorRegistry};
use crate::{
	ConstString,
	behavior::{BehaviorExecution, SubTree, behavior_description::BehaviorDescription},
	tree::BehaviorTree,
	xml::parser::XmlParser,
};
#[allow(unused)]
use crate::{
	behavior::{Behavior, BehaviorKind, BehaviorState, action, condition, control, decorator},
	register_groot2_behavior,
};
#[cfg(feature = "simple_behavior")]
use crate::{
	behavior::{ComplexBhvrTickFn, SimpleBehavior, SimpleBhvrTickFn},
	port::PortList,
};
#[cfg(feature = "mock_behavior")]
use crate::{
	behavior::{MockBehavior, MockBehaviorConfig},
	factory::registry::SubstitutionRule,
	register_behavior,
};
#[allow(unused)]
use alloc::string::String;
use alloc::{boxed::Box, string::ToString, vec::Vec};
use databoard::Databoard;
#[cfg(feature = "mock_behavior")]
use nanoserde::DeJson;
// endregion:   --- modules

// region:      --- BehaviorTreeFactory
/// Factory for creation and modification of [`BehaviorTree`]s.
/// The behaviors are configured via `features`. The following behaviors can be
/// activated via configuration in applications `Cargo.toml`:
/// - Actions:
///   [`AlwaysFailure`](crate::behavior::MockBehavior): feature `always_failure`
///   [`AlwaysRunning`](crate::behavior::MockBehavior): feature `always_running`
///   [`AlwaysSuccess`](crate::behavior::MockBehavior): feature `always_success`
///   [`PopBool`](crate::behavior::action::PopFromQueue): feature `pop_bool`
///   [`PopDouble`](crate::behavior::action::PopFromQueue): feature `pop_double`
///   [`PopInt`](crate::behavior::action::PopFromQueue): feature `pop_int`
///   [`PopString`](crate::behavior::action::PopFromQueue): feature `pop_string`
///   [`Script`](crate::behavior::action::Script): feature `script`
///   [`SetBlackboard`](crate::behavior::action::SetBlackboard): feature `set_blackboard`
///   [`Sleep`](crate::behavior::action::Sleep): feature `sleep`
///   [`UnsetBlackboard`](crate::behavior::action::UnsetBlackboard): feature `unset_blackboard`
/// - Conditions:
///   [`ScriptCondition`](crate::behavior::condition::ScriptCondition): feature `script_condition`
///   [`WasEntryUpdated`](crate::behavior::condition::WasEntryUpdated): feature `was_entry_updated`
/// - Controls:
///   [`AsyncFallback`](crate::behavior::control::Fallback): feature `async_fallback`
///   [`AsyncSequence`](crate::behavior::control::Sequence): feature `async_sequence`
///   [`Fallback`](crate::behavior::control::Fallback): feature `fallback`
///   [`IfThenElse`](crate::behavior::control::IfThenElse): feature `if_then_else`
///   [`Sequence`](crate::behavior::control::Sequence): feature `sequence`
///   [`Parallel`](crate::behavior::control::Parallel): feature `parallel`
///   [`ParallelAll`](crate::behavior::control::ParallelAll): feature `parallel_all`
///   [`ReactiveFallback`](crate::behavior::control::ReactiveFallback): feature `reactive_fallback`
///   [`ReactiveSequence`](crate::behavior::control::ReactiveSequence): feature `reactive_sequence`
///   [`SequenceWithMemory`](crate::behavior::control::SequenceWithMemory): feature `sequence_with_memory`
///   [`Switch2`](crate::behavior::control::Switch): feature `switch2`
///   [`Switch3`](crate::behavior::control::Switch): feature `switch3`
///   [`Switch4`](crate::behavior::control::Switch): feature `switch4`
///   [`Switch5`](crate::behavior::control::Switch): feature `switch5`
///   [`Switch6`](crate::behavior::control::Switch): feature `switch6`
///   [`WhileDoElse`](crate::behavior::control::WhileDoElse): feature `while_do_else`
/// - Decorators:
///   [`Delay`](crate::behavior::decorator::Delay): feature `delay`
///   [`ForceFailure`](crate::behavior::decorator::ForceState): feature `force_failure`
///   [`ForceRunning`](crate::behavior::decorator::ForceState): feature `force_success`
///   [`ForceSuccess`](crate::behavior::decorator::ForceState): feature `force_success`
///   [`Inverter`](crate::behavior::decorator::Inverter): feature `inverter`
///   [`KeepRunningUntilFailure`](crate::behavior::decorator::KeepRunningUntilFailure): feature `keep_running_until_failure`
///   [`LoopBool`](crate::behavior::decorator::Loop): feature `loop_bool`
///   [`LoopDouble`](crate::behavior::decorator::Loop): feature `loop_double`
///   [`LoopInt`](crate::behavior::decorator::Loop): feature `loop_int`
///   [`LoopString`](crate::behavior::decorator::Loop): feature `loop_string`
///   [`Precondition`](crate::behavior::decorator::Precondition): feature `precondition`
///   [`Repeat`](crate::behavior::decorator::Repeat): feature `repeat`
///   [`RetryUntilSuccessful`](crate::behavior::decorator::RetryUntilSuccessful): feature `retry_until_successful`
///   [`RunOnce`](crate::behavior::decorator::RunOnce): feature `run_once`
///   [`SkipUnlessUpdated`](crate::behavior::decorator::EntryUpdated): feature `skip_unless_updated`
///   [`Timeout`](crate::behavior::decorator::Timeout): feature `timeout`
///   [`WaitValueUpdated`](crate::behavior::decorator::EntryUpdated):: feature `wait_value_updated`
/// - For mocking and behavior replacements:
///   [`MockBehavior`](crate::behavior::MockBehavior): feature `mock_behavior`
///
/// Always available is
/// - [`SubTree`]: to enable (sub) trees including the root tree
pub struct BehaviorTreeFactory {
	registry: Box<BehaviorRegistry>,
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

	/// Creates a factory with the configured set of behaviors.
	///
	/// # Errors
	/// - if registration of any of the configured behaviors fails.
	pub fn new() -> Result<Box<Self>, Error> {
		let mut f = Box::new(Self {
			registry: Box::new(BehaviorRegistry::default()),
		});
		// subtree is always available
		f.register_groot2_behavior_type::<SubTree>("SubTree")?;

		// actions
		#[cfg(feature = "always_failure")]
		{
			let config = MockBehaviorConfig {
				return_state: BehaviorState::Failure,
				..Default::default()
			};
			register_behavior!(f, MockBehavior, "AlwaysFailure", config.clone(), PortList::default())?;
		}
		#[cfg(feature = "always_running")]
		{
			let config = MockBehaviorConfig {
				return_state: BehaviorState::Running,
				..Default::default()
			};
			register_behavior!(f, MockBehavior, "AlwaysRunning", config.clone(), PortList::default())?;
		}
		#[cfg(feature = "always_success")]
		{
			let config = MockBehaviorConfig {
				return_state: BehaviorState::Success,
				..Default::default()
			};
			register_behavior!(f, MockBehavior, "AlwaysSuccess", config.clone(), PortList::default())?;
		}
		#[cfg(feature = "script")]
		f.register_groot2_behavior_type::<action::Script>("Script")?;
		#[cfg(feature = "set_blackboard")]
		f.register_groot2_behavior_type::<action::SetBlackboard<String>>("SetBlackboard")?;
		#[cfg(feature = "sleep")]
		f.register_groot2_behavior_type::<action::Sleep>("Sleep")?;
		#[cfg(feature = "pop_bool")]
		f.register_behavior_type::<action::PopFromQueue<bool>>("PopBool")?;
		#[cfg(feature = "pop_double")]
		f.register_behavior_type::<action::PopFromQueue<f64>>("PopDouble")?;
		#[cfg(feature = "pop_int")]
		f.register_behavior_type::<action::PopFromQueue<i32>>("PopInt")?;
		#[cfg(feature = "pop_string")]
		f.register_behavior_type::<action::PopFromQueue<String>>("PopString")?;
		#[cfg(feature = "unset_blackboard")]
		f.register_groot2_behavior_type::<action::UnsetBlackboard<String>>("UnsetBlackboard")?;

		// conditions
		#[cfg(feature = "script_condition")]
		f.register_groot2_behavior_type::<condition::ScriptCondition>("ScriptCondition")?;
		#[cfg(feature = "was_entry_updated")]
		f.register_groot2_behavior_type::<condition::WasEntryUpdated>("WasEntryUpdated")?;

		// controls
		#[cfg(feature = "async_fallback")]
		register_groot2_behavior!(f, control::Fallback, "AsyncFallback", true)?;
		#[cfg(feature = "async_sequence")]
		register_groot2_behavior!(f, control::Sequence, "AsyncSequence", true)?;
		#[cfg(feature = "fallback")]
		f.register_groot2_behavior_type::<control::Fallback>("Fallback")?;
		#[cfg(feature = "if_then_else")]
		f.register_groot2_behavior_type::<control::IfThenElse>("IfThenElse")?;
		#[cfg(feature = "parallel")]
		f.register_groot2_behavior_type::<control::Parallel>("Parallel")?;
		#[cfg(feature = "parallel_all")]
		f.register_groot2_behavior_type::<control::ParallelAll>("ParallelAll")?;
		#[cfg(feature = "reactive_fallback")]
		f.register_groot2_behavior_type::<control::ReactiveFallback>("ReactiveFallback")?;
		#[cfg(feature = "reactive_sequence")]
		f.register_groot2_behavior_type::<control::ReactiveSequence>("ReactiveSequence")?;
		#[cfg(feature = "sequence")]
		f.register_groot2_behavior_type::<control::Sequence>("Sequence")?;
		#[cfg(feature = "sequence_with_memory")]
		f.register_groot2_behavior_type::<control::SequenceWithMemory>("SequenceWithMemory")?;
		#[cfg(feature = "switch2")]
		f.register_groot2_behavior_type::<control::Switch<2>>("Switch2")?;
		#[cfg(feature = "switch3")]
		f.register_groot2_behavior_type::<control::Switch<3>>("Switch3")?;
		#[cfg(feature = "switch4")]
		f.register_groot2_behavior_type::<control::Switch<4>>("Switch4")?;
		#[cfg(feature = "switch5")]
		f.register_groot2_behavior_type::<control::Switch<5>>("Switch5")?;
		#[cfg(feature = "switch6")]
		f.register_groot2_behavior_type::<control::Switch<6>>("Switch6")?;
		#[cfg(feature = "while_do_else")]
		f.register_groot2_behavior_type::<control::WhileDoElse>("WhileDoElse")?;

		// decorators
		#[cfg(feature = "delay")]
		f.register_groot2_behavior_type::<decorator::Delay>("Delay")?;
		#[cfg(feature = "force_failure")]
		register_groot2_behavior!(f, decorator::ForceState, "ForceFailure", BehaviorState::Failure)?;
		#[cfg(feature = "force_running")]
		register_behavior!(f, decorator::ForceState, "ForceRunning", BehaviorState::Running)?;
		#[cfg(feature = "force_success")]
		register_groot2_behavior!(f, decorator::ForceState, "ForceSuccess", BehaviorState::Success)?;
		#[cfg(feature = "inverter")]
		f.register_groot2_behavior_type::<decorator::Inverter>("Inverter")?;
		#[cfg(feature = "keep_running_until_failure")]
		f.register_groot2_behavior_type::<decorator::KeepRunningUntilFailure>("KeepRunningUntilFailure")?;
		#[cfg(feature = "loop_bool")]
		f.register_behavior_type::<decorator::Loop<bool>>("LoopBool")?;
		#[cfg(feature = "loop_double")]
		f.register_groot2_behavior_type::<decorator::Loop<f64>>("LoopDouble")?;
		#[cfg(feature = "loop_int")]
		f.register_behavior_type::<decorator::Loop<i32>>("LoopInt")?;
		#[cfg(feature = "loop_string")]
		f.register_groot2_behavior_type::<decorator::Loop<String>>("LoopString")?;
		#[cfg(feature = "precondition")]
		f.register_groot2_behavior_type::<decorator::Precondition>("Precondition")?;
		#[cfg(feature = "repeat")]
		f.register_groot2_behavior_type::<decorator::Repeat>("Repeat")?;
		#[cfg(feature = "retry_until_successful")]
		f.register_groot2_behavior_type::<decorator::RetryUntilSuccessful>("RetryUntilSuccessful")?;
		#[cfg(feature = "run_once")]
		f.register_groot2_behavior_type::<decorator::RunOnce>("RunOnce")?;
		#[cfg(feature = "timeout")]
		f.register_groot2_behavior_type::<decorator::Timeout>("Timeout")?;
		#[cfg(feature = "skip_unless_updated")]
		register_groot2_behavior!(f, decorator::EntryUpdated, "SkipUnlessUpdated", BehaviorState::Skipped)?;
		#[cfg(feature = "wait_value_updated")]
		register_groot2_behavior!(f, decorator::EntryUpdated, "WaitValueUpdated", BehaviorState::Running)?;

		Ok(f)
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
			Err(err) => Err(Error::Create {
				name: name.into(),
				error: err.to_string().into(),
			}),
		}
	}

	/// Create the named [`BehaviorTree`] from registration using external created blackboard.
	/// # Errors
	/// - if no tree with `name` can be found
	/// - if behaviors or subtrees are missing
	pub fn create_tree_with(&mut self, name: &str, blackboard: &Databoard) -> Result<BehaviorTree, Error> {
		let mut parser = XmlParser::default();
		match parser.create_tree_from_definition(name, &mut self.registry, Some(blackboard)) {
			Ok(root) => Ok(BehaviorTree::new(root, &self.registry)),
			Err(err) => Err(Error::Create {
				name: name.into(),
				error: err.to_string().into(),
			}),
		}
	}

	/// Prints out the list of registered behaviors.
	#[cfg(feature = "std")]
	pub fn list_behaviors(&self) {
		self.registry.list_behaviors();
	}

	/// Register the behavior (sub)trees described by the XML.
	/// # Errors
	/// - on incorrect XML
	/// - if tree description is not in BTCPP v4
	/// - if tree is already registered
	pub fn register_behavior_tree_from_text(&mut self, xml: &str) -> Result<(), Error> {
		#[cfg(feature = "std")]
		{
			let dir = std::env::current_dir()?.to_string_lossy().into();
			match XmlParser::register_document(&mut self.registry, xml, &dir) {
				Ok(()) => Ok(()),
				Err(err) => Err(Error::RegisterXml {
					name: dir,
					error: err.to_string().into(),
				}),
			}
		}
		#[cfg(not(feature = "std"))]
		{
			match XmlParser::register_document(&mut self.registry, xml) {
				Ok(()) => Ok(()),
				Err(err) => Err(Error::RegisterXml {
					name: "inline xml".into(),
					error: err.to_string().into(),
				}),
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
			match XmlParser::register_document(&mut self.registry, &xml, &dir) {
				Ok(()) => Ok(()),
				Err(err) => Err(Error::RegisterXml {
					name: dir,
					error: err.to_string().into(),
				}),
			}
		} else {
			Err(Error::RegisterXml {
				name: file_path.to_string_lossy().into(),
				error: "filepath without parent".into(),
			})
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
						return Err(Error::RegisterLib {
							path: name.into(),
							code: res,
						});
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
	#[cfg(feature = "simple_behavior")]
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

	/// Registers a function as [`BehaviorKind::Action`] or [`BehaviorKind::Condition`] which is using ports.
	/// # Errors
	/// - if a behavior with that `name` is already registered
	#[cfg(feature = "simple_behavior")]
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

	/// Registers a substitution rule for a pattern.
	/// # Errors
	/// - if
	#[cfg(feature = "mock_behavior")]
	#[inline]
	pub fn add_substitution_rule(&mut self, pattern: &str, rule: SubstitutionRule) -> Result<(), Error> {
		self.registry.add_substitution_rule(pattern, rule)
	}

	/// Registers substitution rules using a configuration.
	/// # Errors
	/// - if
	#[cfg(feature = "mock_behavior")]
	pub fn load_substitution_rules_from_json(&mut self, json: &str) -> Result<(), Error> {
		let json: super::json_config::JsonConfig = DeJson::deserialize_json(json)?;
		// std::dbg!(&json);
		for (pattern, rule) in json.substitution_rules {
			self.add_substitution_rule(&pattern, rule)?;
		}
		Ok(())
	}

	/// Deletes all registered a substitution rules.
	/// # Errors
	/// - if
	#[cfg(feature = "mock_behavior")]
	#[inline]
	pub fn clear_substitution_rules(&mut self) {
		self.registry.clear_substitution_rules();
	}
}
// endregion:   --- BehaviorTreeFactory
