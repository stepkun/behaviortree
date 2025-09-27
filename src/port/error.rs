// Copyright © 2025 Stephan Kunz

//! [`behaviortree`](crate) [`Port`](crate::port) errors.

use crate::ConstString;

/// Port errors.
#[non_exhaustive]
pub enum Error {
	/// the port is already defined in list of ports
	AlreadyInPortList {
		/// The port name
		key: ConstString,
	},
	/// Could not convert the str into required T
	CouldNotConvert {
		/// The value that cannot be converted
		value: ConstString,
	},
	/// Pass through errors from databoard
	Databoard {
		/// The databoard error
		source: databoard::Error,
	},
	/// The name for the port is not allowed
	NameNotAllowed {
		/// The invalid port name
		port: ConstString,
	},
	/// Key could not be found
	NotFound {
		/// The key that could not be found
		key: ConstString,
	},
}

/// Only default implementation needed.
impl core::error::Error for Error {
	// fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
	// 		None
	// }

	// fn cause(&self) -> Option<&dyn core::error::Error> {
	// 	self.source()
	// }

	// fn provide<'a>(&'a self, request: &mut core::error::Request<'a>) {}
}

impl core::fmt::Debug for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::AlreadyInPortList { key } => write!(f, "AlreadyInPortList(key: {key})"),
			Self::CouldNotConvert { value } => write!(f, "CouldNotConvert(value: {value})"),
			Self::Databoard { source } => write!(f, "Databoard({source})"),
			Self::NameNotAllowed { port } => write!(f, "NameNotAllowed(port: {port})"),
			Self::NotFound { key } => write!(f, "NotFound(key: {key})"),
		}
	}
}

impl core::fmt::Display for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::AlreadyInPortList { key } => write!(f, "the port {key} is already in the list of defined ports"),
			Self::CouldNotConvert { value } => write!(f, "could not convert '{value}' into wanted type"),
			Self::Databoard { source } => write!(f, "accessing blackboard failed with: {source}"),
			Self::NameNotAllowed { port } => write!(f, "the name {port} is not allowed for a port"),
			Self::NotFound { key } => write!(f, "key {key} could not be found"),
		}
	}
}

impl From<databoard::Error> for Error {
	fn from(source: databoard::Error) -> Self {
		Self::Databoard { source }
	}
}
