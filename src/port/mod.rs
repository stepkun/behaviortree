// Copyright © 2025 Stephan Kunz

//! [`behaviortree`](crate) port module.

pub mod error;
mod port_definition;
mod port_direction;
mod port_list;
#[allow(clippy::module_inception)]
mod port_remappings;

// flatten
pub use port_definition::PortDefinition;
pub use port_direction::PortDirection;
pub use port_list::PortList;
pub use port_remappings::PortRemappings;

// region:      --- modules
use crate::{
	AUTOREMAP, ConstString, FAILURE_IF, ID, NAME, ON_FAILURE, ON_HALTED, ON_SUCCESS, POST, SKIP_IF, SUCCESS_IF, WHILE,
};
use error::Error;
// endregion:   --- modules

// region:		--- globals
// forbidden port names
const FORBIDDEN_PORT_NAMES: &[&str] = &[
	NAME, ID, AUTOREMAP, FAILURE_IF, SUCCESS_IF, SKIP_IF, WHILE, ON_HALTED, ON_FAILURE, ON_SUCCESS, POST,
];
// endregion:	--- globals

// region:      --- helper
/// Check on blackboard pointer.
#[must_use]
pub fn is_bb_pointer(port: &str) -> bool {
	port.starts_with('{') && port.ends_with('}')
}

/// Remove blackboard pointer decorations from port name.
#[must_use]
pub fn strip_bb_pointer(port: &str) -> Option<ConstString> {
	if !is_allowed_port_name(&port[1..]) {
		return None;
	}
	Some(port.strip_prefix('{')?.strip_suffix('}')?.into())
}

/// Create a [`PortDefinition`]
/// # Errors
/// - if the name violates the conventions.
#[allow(clippy::extra_unused_type_parameters)]
pub fn create_port<T>(
	direction: PortDirection,
	type_name: &'static str,
	name: &'static str,
	default: &str,
	description: &'static str,
) -> Result<PortDefinition, Error> {
	if is_allowed_port_name(name) {
		Ok(PortDefinition::new(direction, type_name, name, default, description)?)
	} else {
		Err(Error::NameNotAllowed(name.into()))
	}
}

/// Check a name to be allowed for ports.
#[must_use]
pub fn is_allowed_port_name(name: &str) -> bool {
	if name.is_empty() {
		return false;
	}
	let mut iter = name.chars();
	if let Some(first) = iter.next() {
		if first == '@' {
			if let Some(second) = iter.next() {
				if !second.is_alphabetic() {
					return false;
				}
			} else {
				// it is an '@' without a name
				return false;
			}
		} else if !first.is_alphabetic() {
			return false;
		}

		if FORBIDDEN_PORT_NAMES.contains(&name) {
			return false;
		}
	} else {
		// it is an empty name
		return false;
	}
	true
}
// endregion:   --- helper

// region:		---macros
/// macro for creation of an input port definition
#[macro_export]
macro_rules! input_port {
	// 2 elements
	($tp:ty, $name:expr $(,)?) => {{
		$crate::port::create_port::<$tp>(
			$crate::port::PortDirection::In,
			stringify!($tp),
			$name,
			$crate::EMPTY_STR,
			$crate::EMPTY_STR,
		)
		.expect($crate::SHOULD_NOT_HAPPEN)
	}};
	// 3 elements
	($tp:ty, $name:expr, $default:expr $(,)?) => {
		$crate::port::create_port::<$tp>(
			$crate::port::PortDirection::In,
			stringify!($tp),
			$name,
			&$default.to_string(),
			$crate::EMPTY_STR,
		)
		.expect($crate::SHOULD_NOT_HAPPEN)
	};
	// 4 elements
	($tp:ty, $name:expr, $default:expr, $desc:literal $(,)?) => {
		$crate::port::create_port::<$tp>(
			$crate::port::PortDirection::In,
			stringify!($tp),
			$name,
			&$default.to_string(),
			$desc,
		)
		.expect($crate::SHOULD_NOT_HAPPEN)
	};
}

/// macro for creation of an in/out port definition
#[macro_export]
macro_rules! inout_port {
	// 2 elements
	($tp:ty, $name:expr $(,)?) => {{
		$crate::port::create_port::<$tp>(
			$crate::port::PortDirection::InOut,
			stringify!($tp),
			$name,
			$crate::EMPTY_STR,
			$crate::EMPTY_STR,
		)
		.expect($crate::SHOULD_NOT_HAPPEN)
	}};
	// 3 elements
	($tp:ty, $name:expr, $default:expr $(,)?) => {
		$crate::port::create_port::<$tp>(
			$crate::port::PortDirection::InOut,
			stringify!($tp),
			$name,
			&$default.to_string(),
			$crate::EMPTY_STR,
		)
		.expect($crate::SHOULD_NOT_HAPPEN)
	};
	// 4 elements
	($tp:ty, $name:expr, $default:expr, $desc:literal $(,)?) => {
		$crate::port::create_port::<$tp>(
			$crate::port::PortDirection::InOut,
			stringify!($tp),
			$name,
			&$default.to_string(),
			$desc,
		)
		.expect($crate::SHOULD_NOT_HAPPEN)
	};
}

/// macro for creation of an output port definition
#[macro_export]
macro_rules! output_port {
	// 2 elements
	($tp:ty, $name:expr $(,)?) => {{
		$crate::port::create_port::<$tp>(
			$crate::port::PortDirection::Out,
			stringify!($tp),
			$name,
			$crate::EMPTY_STR,
			$crate::EMPTY_STR,
		)
		.expect($crate::SHOULD_NOT_HAPPEN)
	}};
	// 3 elements
	($tp:ty, $name:expr, $default:expr $(,)?) => {
		$crate::port::create_port::<$tp>(
			$crate::port::PortDirection::Out,
			stringify!($tp),
			$name,
			&$default.to_string(),
			$crate::EMPTY_STR,
		)
		.expect($crate::SHOULD_NOT_HAPPEN)
	};
	// 4 elements
	($tp:ty, $name:expr, $default:expr, $desc:literal $(,)?) => {
		$crate::port::create_port::<$tp>(
			$crate::port::PortDirection::Out,
			stringify!($tp),
			$name,
			&$default.to_string(),
			$desc,
		)
		.expect($crate::SHOULD_NOT_HAPPEN)
	};
}

/// macro for creation of a [`PortList`]
#[macro_export]
macro_rules! port_list {
	($($e:expr),* $(,)?) => {$crate::port::PortList(alloc::vec![$($e),*])};
}
// endregion:	--- macros
