// Copyright © 2025 Stephan Kunz

//! `behaviortree` port module

#[doc(hidden)]
extern crate alloc;

pub mod error;
mod port_definition;
mod port_list;
#[allow(clippy::module_inception)]
mod port_remappings;

// flatten
pub use port_definition::PortDefinition;
pub use port_list::PortList;
pub use port_remappings::ConstPortRemappings;
pub use port_remappings::PortRemappings;

// region:      --- modules
use crate::ConstString;
use error::Error;
// endregion:   --- modules

// region:      --- types
const FORBIDDEN_NAMES: &[&str] = &[
	"name",
	"ID",
	"_autoremap",
	"_failureIf",
	"_successIf",
	"_skipIf",
	"_while",
	"_onHalted",
	"_onFailure",
	"_onSuccess",
	"_post",
];
// endregion:   --- types

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
	type_name: &str,
	name: &str,
	default: &str,
	description: &str,
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

		if FORBIDDEN_NAMES.contains(&name) {
			return false;
		}
	} else {
		// it is an empty name
		return false;
	}
	true
}
// endregion:   --- helper

// region:      --- PortDirection
const INPUT: &str = "Input";
const OUTPUT: &str = "Output";
const INOUT: &str = "InOut";

const INPUT_TYPE: &str = "input_port";
const OUTPUT_TYPE: &str = "output_port";
const INOUT_TYPE: &str = "inout_port";

/// Direction of a `Port`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PortDirection {
	/// Input port
	In,
	/// Output port
	Out,
	/// Bidirecional port
	InOut,
}

impl core::fmt::Display for PortDirection {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl PortDirection {
	/// Get the [`PortDirection`] as str
	#[must_use]
	pub const fn as_str(&self) -> &str {
		match self {
			Self::In => INPUT,
			Self::Out => OUTPUT,
			Self::InOut => INOUT,
		}
	}

	/// Get the [`PortDirection`] as `type_port` str
	#[must_use]
	pub const fn type_str(self) -> &'static str {
		match self {
			Self::In => INPUT_TYPE,
			Self::Out => OUTPUT_TYPE,
			Self::InOut => INOUT_TYPE,
		}
	}
}
// endregion:   --- PortDirection

// region:		---macros
/// macro for creation of an input port definition
#[macro_export]
macro_rules! input_port {
	// 2 elements
	($tp:ty, $name:expr $(,)?) => {{
		$crate::port::create_port::<$tp>($crate::port::PortDirection::In, stringify!($tp), $name, "", "")
			.expect($crate::SHOULD_NOT_HAPPEN)
	}};
	// 3 elements
	($tp:ty, $name:literal, $default:expr $(,)?) => {
		$crate::port::create_port::<$tp>(
			$crate::port::PortDirection::In,
			stringify!($tp),
			$name,
			&$default.to_string(),
			"",
		)
		.expect($crate::SHOULD_NOT_HAPPEN)
	};
	// 4 elements
	($tp:ty, $name:ident, $default:expr, $desc:literal $(,)?) => {
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
		$crate::port::create_port::<$tp>($crate::port::PortDirection::InOut, stringify!($tp), $name, "", "")
			.expect($crate::SHOULD_NOT_HAPPEN)
	}};
	// 3 elements
	($tp:ty, $name:literal, $default:expr $(,)?) => {
		$crate::port::create_port::<$tp>(
			$crate::port::PortDirection::InOut,
			stringify!($tp),
			$name,
			&$default.to_string(),
			"",
		)
		.expect($crate::SHOULD_NOT_HAPPEN)
	};
	// 4 elements
	($tp:ty, $name:ident, $default:expr, $desc:literal $(,)?) => {
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
		$crate::port::create_port::<$tp>($crate::port::PortDirection::Out, stringify!($tp), $name, "", "")
			.expect($crate::SHOULD_NOT_HAPPEN)
	}};
	// 3 elements
	($tp:ty, $name:literal, $default:expr $(,)?) => {
		$crate::port::create_port::<$tp>(
			$crate::port::PortDirection::Out,
			stringify!($tp),
			$name,
			&$default.to_string(),
			"",
		)
		.expect($crate::SHOULD_NOT_HAPPEN)
	};
	// 4 elements
	($tp:ty, $name:ident, $default:expr, $desc:literal $(,)?) => {
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
