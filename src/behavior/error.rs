// Copyright © 2024 Stephan Kunz
//! `behaviortree` behavior errors

#[doc(hidden)]
extern crate alloc;

// region		--- modules
use super::BehaviorState;
use crate::ConstString;
// endregion:	--- modules

/// Behavior errors.
#[non_exhaustive]
pub enum Error {
	/// Error in structural composition of a behaviors children
	Composition {
		/// The textual error message.
		txt: ConstString,
	},
	/// Pass through errors from databoard
	Databoard {
		/// The source error
		source: databoard::Error,
	},
	/// Pass through errors from nanoserde
	Nanoserde {
		/// The source error
		source: nanoserde::DeJsonErr,
	},
	/// Attribute is not a pre or post condition
	NoCondition {
		/// The attribute
		value: ConstString,
	},
	/// Value is not a boolean type
	NotABool {
		/// The non boolean value
		value: ConstString,
	},
	/// Parsing error during type conversion
	ParseError {
		/// The non parseable value
		value: ConstString,
		/// The source of this value
		src: ConstString,
	},
	/// Pass through errors from `core::num`
	ParseInt {
		/// The source error
		source: core::num::ParseIntError,
	},
	/// Type mismatch between port definiton and found value
	ParsePortValue {
		/// The ports name
		port: ConstString,
		/// The wanted data type
		typ: ConstString,
	},
	/// Pass through errors from `crate::port`
	Port {
		/// The port error
		source: crate::port::error::Error,
	},
	/// Port has not been defined in behavior
	PortNotDeclared {
		/// Name of the port
		port: ConstString,
		/// Affected behavior
		behavior: ConstString,
	},
	/// Pass through errors from tinyscript
	Scripting {
		/// The scripting eror
		source: tinyscript::Error,
	},
	/// An invalid [`BehaviorState`] is reached
	State {
		/// The affected behavior
		behavior: ConstString,
		/// The invalid state
		state: BehaviorState,
	},
	/// Unable to set the pre or post condition
	UnableToSetCondition {
		/// Te condition thatcannot be set
		value: ConstString,
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
			Self::Composition { txt } => write!(f, "Composition({txt})"),
			Self::Databoard { source } => write!(f, "Databoard({source})"),
			Self::Nanoserde { source } => write!(f, "Nanoserde({source})"),
			Self::NoCondition { value } => write!(f, "NoCondition(value: {value})"),
			Self::NotABool { value } => write!(f, "NotABool(value: {value})"),
			Self::ParseError { value, src } => write!(f, "ParseError(value: {value}, src: {src})"),
			Self::ParseInt { source } => write!(f, "ParseInt({source})"),
			Self::ParsePortValue { port, typ } => write!(f, "ParsePort(port: {port}, type: {typ})"),
			Self::Port { source } => write!(f, "Port({source})"),
			Self::PortNotDeclared { port, behavior } => write!(f, "PortNotDeclared(port: {port}, behavior: {behavior})"),
			Self::Scripting { source } => write!(f, "Scripting({source})"),
			Self::State { behavior, state } => write!(f, "State(behavior: {behavior}, state: {state})"),
			Self::UnableToSetCondition { value } => write!(f, "UnableToSetCondition(value: {value})"),
		}
	}
}

impl core::fmt::Display for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::Composition { txt } => write!(f, "behavior composition error: {txt}"),
			Self::Databoard { source } => write!(f, "a blackboard error occured: {source}"),
			Self::Nanoserde { source } => write!(f, "a deserialization error occured: {source}"),
			Self::NoCondition { value } => write!(f, "the attribute '{value}' is no pre or post condition"),
			Self::NotABool { value } => write!(f, "value {value} is not a boolean type"),
			Self::ParseError { value, src } => write!(f, "could not parse value '{value}' in {src}"),
			Self::ParseInt { source } => write!(f, "could not parse int value: {source}"),
			Self::ParsePortValue { port, typ } => {
				write!(f, "could not parse value for port {port} into specified type {typ}")
			}
			Self::Port { source } => write!(f, "a port error occured: {source}"),
			Self::PortNotDeclared { port, behavior } => write!(f, "port {port} is not declared in behavior {behavior}"),
			Self::Scripting { source } => write!(f, "a scripting error occured: {source}"),
			Self::State { behavior, state } => {
				write!(f, "child node of  {behavior} returned state {state} when not allowed")
			}
			Self::UnableToSetCondition { value } => write!(f, "unable to set the pre or post condition {value})"),
		}
	}
}

impl From<crate::port::error::Error> for Error {
	fn from(source: crate::port::error::Error) -> Self {
		Self::Port { source }
	}
}

impl From<core::num::ParseIntError> for Error {
	fn from(source: core::num::ParseIntError) -> Self {
		Self::ParseInt { source }
	}
}
impl From<databoard::Error> for Error {
	fn from(source: databoard::Error) -> Self {
		Self::Databoard { source }
	}
}

impl From<tinyscript::Error> for Error {
	fn from(source: tinyscript::Error) -> Self {
		Self::Scripting { source }
	}
}

impl From<nanoserde::DeJsonErr> for Error {
	fn from(source: nanoserde::DeJsonErr) -> Self {
		Self::Nanoserde { source }
	}
}
