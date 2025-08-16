// Copyright © 2025 Stephan Kunz

//! [`behaviortree`](crate) [`PortDefinition`] implementation.

// region:      --- modules
use crate::ConstString;

use super::{error::Error, is_allowed_port_name, port_direction::PortDirection};
// endregion:   --- modules

// region:      --- PortDefinition
/// A [`PortDefinition`], which is used for configuration.
#[derive(Clone, Debug)]
pub struct PortDefinition {
	/// Direction of the port.
	direction: PortDirection,
	/// Data type of the port.
	/// This is a `&'static str`, created by the port creation macro.
	type_name: &'static str,
	/// Name of the port.
	/// This has to be a `&'static str`.
	name: &'static str,
	/// Default value for the port.
	default_value: ConstString,
	/// Description of the port.
	/// This has to be a `&'static str`.
	description: &'static str,
}

impl PortDefinition {
	/// Constructor
	/// # Errors
	/// - if the name violates the conventions.
	pub fn new(
		direction: PortDirection,
		type_name: &'static str,
		name: &'static str,
		default_value: &str,
		description: &'static str,
	) -> Result<Self, Error> {
		if is_allowed_port_name(name) {
			Ok(Self {
				direction,
				type_name,
				name,
				default_value: default_value.into(),
				description,
			})
		} else {
			Err(Error::NameNotAllowed(name.into()))
		}
	}

	/// Get the [`PortDefinition`]s name.
	#[must_use]
	pub const fn name(&self) -> &'static str {
		self.name
	}

	/// Get the [`PortDefinition`]s direction.
	#[must_use]
	pub const fn direction(&self) -> &PortDirection {
		&self.direction
	}

	/// Get the default value.
	#[must_use]
	pub fn default_value(&self) -> Option<&ConstString> {
		if self.default_value.is_empty() {
			None
		} else {
			Some(&self.default_value)
		}
	}

	#[must_use]
	pub(crate) const fn type_name(&self) -> &'static str {
		self.type_name
	}

	#[must_use]
	pub(crate) const fn description(&self) -> &'static str {
		self.description
	}
}
// endregion:   --- PortDefinition
