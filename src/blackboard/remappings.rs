// Copyright © 2025 Stephan Kunz
//! [`behaviortree`](crate) [`PortRemappings`] implementation.

// region:      --- modules
use alloc::{string::String, vec::Vec};
use core::ops::{Deref, DerefMut};

use crate::ConstString;

use super::error::Error;
// endregion:   --- modules

// region:		--- types
/// An immutable remapping entry.
type RemappingEntry = (ConstString, ConstString);
// endregion:   --- types

// region:		--- PortRemappings
/// Mutable remapping list.
#[derive(Clone, Debug, Default)]
#[repr(transparent)]
pub struct Remappings(Vec<RemappingEntry>);

impl Deref for Remappings {
	type Target = Vec<RemappingEntry>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Remappings {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Remappings {
	/// Add an entry to the [`PortRemappings`].
	/// The original name is a `&'static str` as provided by
	/// [`PortDefinition`](crate::port::port_definition::PortDefinition)
	/// # Errors
	/// - if entry already exists
	pub fn add(&mut self, name: &'static str, remapped_name: impl Into<ConstString>) -> Result<(), Error> {
		for (original, _) in &self.0 {
			if original.as_ref() == name {
				return Err(Error::AlreadyInRemappings(name.into()));
			}
		}
		self.0.push((name.into(), remapped_name.into()));
		Ok(())
	}

	/// Add an entry to the [`PortRemappings`].
	/// Already existing values will be overwritten
	pub fn overwrite(&mut self, name: &str, remapped_name: impl Into<ConstString>) {
		for (original, old_value) in &mut self.0 {
			if original.as_ref() == name {
				// replace value
				*old_value = remapped_name.into();
				return;
			}
		}
		// create if not existent
		self.0.push((name.into(), remapped_name.into()));
	}

	/// Lookup the remapped name.
	#[must_use]
	pub fn find(&self, name: &str) -> Option<ConstString> {
		for (original, remapped) in &self.0 {
			if original.as_ref() == name {
				// is the shortcut '{=}' used?
				return if remapped.as_ref() == "{=}" {
					Some((String::from("{") + name + "}").into())
				} else {
					Some(remapped.clone())
				};
			}
		}
		None
	}

	/// Optimize for size
	pub fn shrink(&mut self) {
		self.0.shrink_to_fit();
	}
}
// endregion:   --- PortRemappings
