// Copyright © 2025 Stephan Kunz

//! [`behaviortree`](crate) port module.

pub mod error;
mod port_definition;
mod port_direction;
mod port_list;

// flatten
pub use port_definition::PortDefinition;
pub use port_direction::PortDirection;
pub use port_list::PortList;

use crate::{AUTOREMAP, FAILURE_IF, ID, NAME, ON_FAILURE, ON_HALTED, ON_SUCCESS, POST, SKIP_IF, SUCCESS_IF, WHILE};
use error::Error;

// forbidden port names
const FORBIDDEN_PORT_NAMES: &[&str] = &[
	NAME, ID, AUTOREMAP, FAILURE_IF, SUCCESS_IF, SKIP_IF, WHILE, ON_HALTED, ON_FAILURE, ON_SUCCESS, POST,
];

// region:   	--- helper
/// Create a [`PortDefinition`]
/// # Errors
/// - if the name violates the conventions.
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
		Err(Error::NameNotAllowed { port: name.into() })
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
		.expect("macro input_port case 1 failed")
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
		.expect("macro input_port case 2 failed")
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
		.expect("macro input_port case 3 failed")
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
		.expect("macro inout_port case 1 failed")
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
		.expect("macro inout_port case 2 failed")
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
		.expect("macro inout_port case 3 failed")
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
		.expect("macro output_port case 1 failed")
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
		.expect("macro output_port case 2 failed")
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
		.expect("macro output_port case 3 failed")
	};
}

/// macro for creation of a [`PortList`]
#[macro_export]
macro_rules! port_list {
	($($e:expr),* $(,)?) => {$crate::port::PortList(alloc::vec![$($e),*])};
}
// endregion:	--- macros

#[cfg(test)]
mod tests {
	use crate::port::PortDirection;

	use super::*;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<error::Error>();
		is_normal::<PortDefinition>();
		is_normal::<PortDirection>();
		is_normal::<PortList>();
	}
}
