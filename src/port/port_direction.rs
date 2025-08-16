// Copyright © 2025 Stephan Kunz

//! [`behaviortree`](crate) [`PortDirection`] implementation.

// region:      --- PortDirection
// str constants for function `as_str()`.
const INPUT: &str = "Input";
const OUTPUT: &str = "Output";
const INOUT: &str = "InOut";

// These values are used in communication with Groot2 when creating the tree xml.
// See function `type_str()` below.
const INPUT_TYPE: &str = "input_port";
const OUTPUT_TYPE: &str = "output_port";
const INOUT_TYPE: &str = "inout_port";

/// Direction of a [`Port`](crate::port).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PortDirection {
	/// Input port
	In,
	/// Output port
	Out,
	/// Bidirectional port
	InOut,
}

impl core::fmt::Display for PortDirection {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl PortDirection {
	/// Get the [`PortDirection`] as &str.
	#[must_use]
	pub const fn as_str(&self) -> &str {
		match self {
			Self::In => INPUT,
			Self::Out => OUTPUT,
			Self::InOut => INOUT,
		}
	}

	/// Get the [`PortDirection`] as `<type>_port` str.
	///
	/// Used when creating the tree xml. See: [`XmlCreator`](crate::xml::creator::XmlCreator).
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
