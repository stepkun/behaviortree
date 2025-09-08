// Copyright © 2025 Stephan Kunz
//! Blackboard errors.

#[doc(hidden)]
extern crate alloc;

// region		--- modules
use crate::ConstString;
use thiserror::Error;
// endregion:	--- modules

// region:		--- Error
/// `blackboard` error type
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
	/// Enry already in [`Remappings`](crate::blackboard::remappings::Remappings)
	#[error("name [{0}] already in remapping list")]
	AlreadyInRemappings(ConstString),
	/// Entry is not in `Blackboard`.
	#[error("couldn't find entry [{0}] in blackboard")]
	NotFound(ConstString),
	/// Entry is not in `Blackboard`.
	#[error("couldn't find entry [{0}] in blackboard of [{1}]")]
	NotFoundIn(ConstString, ConstString),
	/// Entry has other type than expected.
	#[error("entry [{0}] has a different type")]
	WrongType(ConstString),
	/// Type mismatch between port definiton and found value
	#[error("could not parse value for port [{0}] into specified type [{1}]")]
	ParsePortValue(ConstString, ConstString),
	/// Port is not defined.
	#[error("couldn't find port [{0}]")]
	Port(ConstString),
}
// region:		--- Error
