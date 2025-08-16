// Copyright © 2025 Stephan Kunz

//! [`behaviortree`](crate) [`PortRemappings`] and [`ConstPortRemappings`] implementation.

// region:      --- modules
use alloc::{boxed::Box, string::String, vec::Vec};
use core::ops::{Deref, DerefMut};

use crate::ConstString;

use super::error::Error;
// endregion:   --- modules

// region:		--- types
/// An immutable remapping entry.
type RemappingEntry = (ConstString, ConstString);
// endregion:   --- types

// region:		--- ConstPortRemappings
/// An immutable remapping list.
///
/// Use [`PortRemappings`] to build a remapping list and convert it into
/// an immutable list if it will never change after creation.
#[derive(Clone, Debug, Default)]
pub struct ConstPortRemappings(Box<[RemappingEntry]>);

impl Deref for ConstPortRemappings {
	type Target = [RemappingEntry];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for ConstPortRemappings {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl From<PortRemappings> for ConstPortRemappings {
	fn from(remappings: PortRemappings) -> Self {
		Self(remappings.0.into_boxed_slice())
	}
}

impl ConstPortRemappings {
	/// Lookup the remapped name.
	#[must_use]
	pub fn find(&self, name: &ConstString) -> Option<ConstString> {
		for (original, remapped) in &self.0 {
			if original == name {
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
}
// endregion:   --- ConstPortRemappings

// region:		--- PortRemappings
/// Mutable remapping list.
#[derive(Clone, Debug, Default)]
pub struct PortRemappings(Vec<RemappingEntry>);

impl Deref for PortRemappings {
	type Target = Vec<RemappingEntry>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for PortRemappings {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl From<ConstPortRemappings> for PortRemappings {
	fn from(remappings: ConstPortRemappings) -> Self {
		Self(remappings.0.into_vec())
	}
}

impl PortRemappings {
	/// Add an entry to the [`PortRemappings`].
	/// # Errors
	/// - if entry already exists
	pub fn add(&mut self, name: &ConstString, remapped_name: &ConstString) -> Result<(), Error> {
		for (original, _) in &self.0 {
			if original == name {
				return Err(Error::AlreadyInRemappings(name.clone()));
			}
		}
		self.0.push((name.clone(), remapped_name.clone()));
		Ok(())
	}

	/// Add an entry to the [`PortRemappings`].
	/// Already existing values will be overwritten
	pub fn overwrite(&mut self, key: &ConstString, value: &ConstString) {
		for (original, old_value) in &mut self.0 {
			if original == key {
				// replace value
				*old_value = value.clone();
				return;
			}
		}
		// create if not existent
		self.0.push((key.clone(), value.clone()));
	}

	// /// Lookup the remapped name.
	// #[must_use]
	// pub fn find(&self, name: &ConstString) -> Option<ConstString> {
	// 	for (original, remapped) in &self.0 {
	// 		if original == name {
	// 			// is the shortcut '{=}' used?
	// 			return if remapped.as_ref() == "{=}" {
	// 				Some((String::from("{") + name + "}").into())
	// 			} else {
	// 				Some(remapped.clone())
	// 			};
	// 		}
	// 	}
	// 	None
	// }

	// /// Optimize for size
	// pub fn shrink(&mut self) {
	// 	self.0.shrink_to_fit();
	// }
}
// endregion:   --- PortRemappings
