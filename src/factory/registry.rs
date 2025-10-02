// Copyright © 2025 Stephan Kunz

//! [`BehaviorRegistry`] library
//!

#[doc(hidden)]
#[cfg(feature = "std")]
extern crate std;

use core::ops::Range;

// region:      --- modules
use alloc::{boxed::Box, collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
#[cfg(feature = "std")]
use libloading::Library;
use tinyscript::Runtime;

use crate::{
	BehaviorExecution, ConstString,
	behavior::{
		BehaviorCreationFn, BehaviorPtr, TestBehavior, TestBehaviorConfig, behavior_description::BehaviorDescription,
	},
	port::PortDirection,
};

use super::error::Error;

#[cfg(doc)]
use super::BehaviorTreeFactory;
// endregion:   --- modules

// region:		--- SubstitutionRule
/// Variants of substitution rules
#[derive(Clone, Debug)]
pub enum SubstitutionRule {
	/// Rule is a String type, replacing a behavior with some other behavior
	/// using the behavior ID for finding the other behavior.
	StringRule(ConstString),
	/// Rule creates a [`TestBehavior`] with the given configuration.
	ConfigRule(TestBehaviorConfig),
}
// endregion:	--- SubstitutionRule

// region:     --- TreeNodesModelEntry
/// A `TreeNodesModel` entry.
#[derive(Debug)]
pub(crate) struct TreeNodesModelEntry {
	pub(crate) _port_type: PortDirection,
	pub(crate) key: ConstString,
	pub(crate) remapping: ConstString,
}
// endregion:	--- TreeNodesModelEntry

// region:     --- BehaviorRegistry
/// A registry for behaviors used by the [`BehaviorTreeFactory`](crate::factory::BehaviorTreeFactory) for creation of behavior trees.
#[derive(Default)]
pub struct BehaviorRegistry {
	/// [`BTreeMap`] of available behavior creation functions.
	/// The key is the name stored in the [`BehaviorDescription`].
	behaviors: BTreeMap<ConstString, (BehaviorDescription, Arc<BehaviorCreationFn>)>,
	/// [`BTreeMap`] of registered behavior tree definitions.
	tree_definitions: BTreeMap<ConstString, (ConstString, Range<usize>)>,
	/// `TreNodesModel` remappings. The key is combined from behaviors type and ID.
	tree_nodes_models: BTreeMap<ConstString, TreeNodesModelEntry>,
	/// Substitution rules
	substitution_rules: BTreeMap<ConstString, SubstitutionRule>,
	/// Main tree ID
	main_tree_id: Option<ConstString>,
	/// Scripting runtime
	runtime: Box<Runtime>,
	/// List of loaded libraries.
	/// Every tree must keep a reference to its needed libraries to keep the libraries in memory
	/// until end of programm.
	#[cfg(feature = "std")]
	libraries: Vec<Arc<Library>>,
}

impl BehaviorRegistry {
	/// Add a behavior to the registry
	/// # Errors
	/// - if the behavior entry already exists
	pub fn add_behavior<F>(&mut self, bhvr_description: BehaviorDescription, bhvr_creation_fn: F) -> Result<(), Error>
	where
		F: Fn() -> BehaviorPtr + Send + Sync + 'static,
	{
		if self
			.behaviors
			.contains_key(bhvr_description.name())
		{
			return Err(Error::AlreadyRegistered {
				name: bhvr_description.name().clone(),
			});
		}
		self.behaviors.insert(
			bhvr_description.name().clone(),
			(bhvr_description, Arc::from(bhvr_creation_fn)),
		);
		Ok(())
	}

	pub(crate) const fn behaviors(&self) -> &BTreeMap<ConstString, (BehaviorDescription, Arc<BehaviorCreationFn>)> {
		&self.behaviors
	}

	/// The Library must be kept in storage until the behaviort tree is destroyed.
	/// Therefore the library is stored in the behavior registry and later a cloned
	/// reference is handed over to every created tree.
	#[cfg(feature = "std")]
	pub fn add_library(&mut self, library: Library) {
		self.libraries.push(Arc::new(library));
	}

	/// Registers a substitution rule for a pattern.
	/// # Errors
	/// - if
	pub fn add_substitution_rule(&mut self, pattern: &str, rule: SubstitutionRule) -> Result<(), Error> {
		self.substitution_rules
			.insert(pattern.into(), rule);
		Ok(())
	}

	/// Deletes all registered a substitution rules.
	/// # Errors
	/// - if
	#[inline]
	pub fn clear_substitution_rules(&mut self) {
		self.substitution_rules.clear();
	}

	/// Adds a `TreeNodesModelEntry` to the registy.
	/// # Errors
	/// - if an entry with that key already exists.
	pub(crate) fn add_tree_nodes_model_entry(&mut self, key: ConstString, entry: TreeNodesModelEntry) -> Result<(), Error> {
		if self.tree_nodes_models.contains_key(&key) {
			return Err(Error::AlreadyRegistered { name: key });
		}
		self.tree_nodes_models.insert(key, entry);
		Ok(())
	}

	pub(crate) const fn tree_nodes_models(&self) -> &BTreeMap<ConstString, TreeNodesModelEntry> {
		&self.tree_nodes_models
	}

	/// Set the main tree id
	pub fn set_main_tree_id(&mut self, id: &str) {
		self.main_tree_id = Some(id.into());
	}

	/// Clear registered behavior trees.
	///
	/// Clears only the registered trees, not the registered behaviors.
	/// In case you want to clear everything, use a new factory.
	pub fn clear_registered_trees(&mut self) {
		// delete the main tree id
		self.main_tree_id = None;
		// remove tree definitions
		self.tree_definitions.clear();
		// @TODO: What about the libraries???
	}

	/// Get the main tree id
	#[must_use]
	pub fn main_tree_id(&self) -> Option<ConstString> {
		self.main_tree_id.clone()
	}

	/// Add a behavior tree definition to the registry.
	/// # Errors
	/// - if the behavior tree definition is already registered.
	pub(crate) fn add_tree_defintion(
		&mut self,
		id: &str,
		tree_definition: ConstString,
		range: Range<usize>,
	) -> Result<(), Error> {
		let key: ConstString = id.into();
		if let alloc::collections::btree_map::Entry::Vacant(e) = self.tree_definitions.entry(key) {
			e.insert((tree_definition, range));
			Ok(())
		} else {
			Err(Error::AlreadyRegistered { name: id.into() })
		}
	}

	/// Fetch a behavior creation function from the registry.
	/// # Errors
	/// - if the behavior is not found in the registry
	#[allow(clippy::option_if_let_else)]
	pub(crate) fn fetch_behavior(
		&self,
		id: &str,
		path: &str,
	) -> Result<(BehaviorDescription, Box<dyn BehaviorExecution>), Error> {
		// look for a substitution rule
		// the first matching rule will be used
		let mut result: Option<SubstitutionRule> = None;
		for (pattern, rule) in &self.substitution_rules {
			// #[cfg(feature = "std")]
			// std::dbg!(pattern, path);
			let sub_patterns = pattern.split('*');
			// find each sub pattern in the right sequence
			let mut pos = 0_usize;
			let mut found = true;
			for p in sub_patterns {
				// #[cfg(feature = "std")]
				// std::dbg!(p);
				if let Some(pattern_pos) = path[pos..].find(p) {
					pos = pattern_pos;
				} else {
					found = false;
					break;
				}
			}
			if found {
				result = Some(rule.clone());
				break;
			}
		}

		if let Some(substitution) = result {
			match substitution {
				SubstitutionRule::StringRule(id) => {
					// fetch from registry
					self.behaviors.get(&id).map_or_else(
						|| Err(Error::NotRegistered { name: id }),
						|(desc, creation_fn)| {
							let bhvr = creation_fn();
							Ok((desc.clone(), bhvr))
						},
					)
				}
				SubstitutionRule::ConfigRule(test_behavior_config) => {
					// find original entry for description info
					self.behaviors.get(id).map_or_else(
						|| Err(Error::NotRegistered { name: id.into() }),
						|(desc, creation_fn)| {
							let old_behavior = creation_fn();
							let port_list = old_behavior.static_provided_ports();
							// create a TestBehavior instead of original behavior
							let bhvr_fn = TestBehavior::creation_fn(test_behavior_config.clone(), port_list);
							Ok((desc.clone(), bhvr_fn()))
						},
					)
				}
			}
		} else {
			// fetch from registry
			self.behaviors.get(id).map_or_else(
				|| Err(Error::NotRegistered { name: id.into() }),
				|(desc, creation_fn)| {
					let bhvr = creation_fn();
					Ok((desc.clone(), bhvr))
				},
			)
		}
	}

	#[must_use]
	pub(crate) fn find_tree_definition(&self, name: &str) -> Option<(ConstString, Range<usize>)> {
		self.tree_definitions.get(name).cloned()
	}

	/// Prints out the list of registered behaviors
	#[cfg(feature = "std")]
	pub fn list_behaviors(&self) {
		let iter = self.behaviors.iter();
		for (key, _) in iter {
			std::println!("{key}");
		}
		std::println!();
	}

	/// Get a reference to the registered libraries
	#[cfg(feature = "std")]
	#[must_use]
	pub(crate) const fn libraries(&self) -> &Vec<Arc<Library>> {
		&self.libraries
	}

	/// Get the name list of registered (sub)trees
	#[must_use]
	pub fn registered_behavior_trees(&self) -> Vec<ConstString> {
		let mut res = Vec::new();
		for id in self.tree_definitions.keys() {
			res.push(id.clone());
		}
		res
	}

	/// Access the runtime.
	#[must_use]
	pub const fn runtime(&self) -> &Runtime {
		&self.runtime
	}

	/// Access the runtime mutable.
	pub const fn runtime_mut(&mut self) -> &mut Runtime {
		&mut self.runtime
	}

	pub(crate) fn register_enum_tuple(&mut self, key: &str, value: i8) -> Result<(), Error> {
		self.runtime.register_enum_tuple(key, value)?;
		Ok(())
	}
}
// endregion:   --- BehaviorRegistry
