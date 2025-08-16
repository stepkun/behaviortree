// Copyright © 2025 Stephan Kunz

//! [`behaviortree`](crate) [`PortList`] implementation.

// region:      --- modules
use alloc::{string::String, vec::Vec};
use core::ops::{Deref, DerefMut};

use crate::ConstString;

use super::{PortDefinition, error::Error};
// endregion:   --- modules

// region:      --- PortList
/// List of ports
/// The `PortList` is not using a `BTreeMap` but a `Vec` due to
/// a `BTreeMap` needs more space than a `Vec` and search performance is not an issue
#[derive(Clone, Debug, Default)]
#[repr(transparent)]
pub struct PortList(pub Vec<PortDefinition>);

impl Deref for PortList {
	type Target = Vec<PortDefinition>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for PortList {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl PortList {
	/// Add an entry to the [`PortList`]
	/// # Errors
	/// - if entry already exists
	pub fn add(&mut self, port_definition: PortDefinition) -> Result<(), Error> {
		for entry in &self.0 {
			if entry.name() == port_definition.name() {
				return Err(Error::AlreadyInPortList(entry.name().clone()));
			}
		}
		self.0.push(port_definition);
		Ok(())
	}

	/// Create a list of the `Port` names in the list
	#[must_use]
	pub fn entries(&self) -> ConstString {
		let comma = false;
		let mut result = String::new();
		for entry in &self.0 {
			if comma {
				result += ", ";
			}
			result += entry.name();
		}
		result.into()
	}

	/// Lookup a [`PortDefinition`].
	#[must_use]
	pub fn find(&self, name: &str) -> Option<PortDefinition> {
		for entry in &self.0 {
			if entry.name().as_ref() == name {
				return Some(entry.clone());
			}
		}
		None
	}
}
// endregion:	--- PortList
