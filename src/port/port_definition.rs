// Copyright © 2025 Stephan Kunz

//! `behaviortree` `PortDefintion` implementation

#[doc(hidden)]
extern crate alloc;

// region:      --- modules
use crate::ConstString;

use super::{PortDirection, error::Error, is_allowed_port_name};
// endregion:   --- modules

// region:      --- PortDefinition
/// A static [`PortDefinition`], which is used for configuration.
/// Access to members is public within crate to maximize performance
#[derive(Clone, Debug)]
pub struct PortDefinition {
    /// Direction of the port.
    direction: PortDirection,
    /// Data type of the port.
    type_name: ConstString,
    /// Name of the port.
    name: ConstString,
    /// Default value for the port.
    default_value: ConstString,
    /// Description of the port.
    description: ConstString,
}

impl PortDefinition {
    /// Constructor
    /// # Errors
    /// - if the name violates the conventions.
    pub fn new(
        direction: PortDirection,
        type_name: &str,
        name: &str,
        default_value: &str,
        description: &str,
    ) -> Result<Self, Error> {
        if is_allowed_port_name(name) {
            Ok(Self {
                direction,
                type_name: type_name.into(),
                name: name.into(),
                default_value: default_value.into(),
                description: description.into(),
            })
        } else {
            Err(Error::NameNotAllowed(name.into()))
        }
    }

    /// Get the [`PortDefinition`]s name.
    #[must_use]
    pub const fn name(&self) -> &ConstString {
        &self.name
    }

    /// Get the [`PortDefinition`]s direction.
    #[must_use]
    pub const fn direction(&self) -> &PortDirection {
        &self.direction
    }

    /// Get the default value.
    #[must_use]
    pub fn default_value(&self) -> Option<ConstString> {
        if self.default_value.is_empty() {
            None
        } else {
            Some(self.default_value.clone())
        }
    }

    pub(crate) const fn type_name(&self) -> &ConstString {
        &self.type_name
    }

    #[allow(unused)]
    pub(crate) const fn description(&self) -> &ConstString {
        &self.description
    }
}
// endregion:   --- PortDefinition
