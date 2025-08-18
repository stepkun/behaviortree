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
	ConstString,
	behavior::{BehaviorCreationFn, BehaviorDescription, BehaviorPtr},
};

use super::error::Error;

#[cfg(doc)]
use super::BehaviorTreeFactory;
// endregion:   --- modules

// region:     --- BehaviorRegistry
/// A registry for behaviors used by the [`BehaviorTreeFactory`] for creation of behavior trees.
#[derive(Default)]
pub struct BehaviorRegistry {
	/// [`BTreeMap`] of available behavior creation functions.
	/// The key is the name stored in the [`BehaviorDescription`].
	behaviors: BTreeMap<ConstString, (BehaviorDescription, Arc<BehaviorCreationFn>)>,
	/// [`BTreeMap`] of registered behavior tree definitions.
	tree_definitions: BTreeMap<ConstString, (ConstString, Range<usize>)>,
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
	/// - if the entry already exists
	pub fn add_behavior<F>(&mut self, bhvr_description: BehaviorDescription, bhvr_creation_fn: F) -> Result<(), Error>
	where
		F: Fn() -> BehaviorPtr + Send + Sync + 'static,
	{
		if self
			.behaviors
			.contains_key(bhvr_description.name())
		{
			return Err(Error::BehaviorAlreadyRegistered(bhvr_description.name().clone()));
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

	/// Set the main tree id
	pub fn set_main_tree_id(&mut self, id: &str) {
		self.main_tree_id = Some(id.into());
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
			Err(Error::SubtreeAlreadyRegistered(id.into()))
		}
	}

	/// Fetch a behavior creation function from the registry.
	/// # Errors
	/// - if the behavior is not found in the registry
	pub(crate) fn fetch(&self, id: &str) -> Result<(BehaviorDescription, Arc<BehaviorCreationFn>), Error> {
		self.behaviors
			.get(id)
			.map_or_else(|| Err(Error::BehaviorNotRegistered(id.into())), |value| Ok(value.clone()))
	}

	#[must_use]
	pub(crate) fn find_tree_definition(&self, name: &str) -> Option<(ConstString, Range<usize>)> {
		self.tree_definitions.get(name).cloned()
	}

	/// Prints out the list of registered behaviors
	pub fn list_behaviors(&self) {
		#[cfg(feature = "std")]
		let iter = self.behaviors.iter();
		#[cfg(feature = "std")]
		for (key, _) in iter {
			std::println!("{key}");
		}
		#[cfg(feature = "std")]
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
