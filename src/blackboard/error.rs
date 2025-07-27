// Copyright © 2025 Stephan Kunz

//! `dimas-behavior` Blackboard errors

#[doc(hidden)]
extern crate alloc;

// region		--- modules
use crate::ConstString;
use thiserror::Error;
// endregion:	--- modules

// region:		--- Error
/// `blackboard` error type
#[derive(Error, Debug)]
pub enum Error {
	/// Entry is not in `Blackboard`.
	#[error("Couldn't find entry [{0}] in blackboard")]
	NotFound(ConstString),
	/// Entry is not in `Blackboard`.
	#[error("Couldn't find entry [{0}] in blackboard of [{1}]")]
	NotFoundIn(ConstString, ConstString),
	/// Entry has other type than expected.
	#[error("Entry [{0}] has a different type")]
	WrongType(ConstString),
	/// Type mismatch between port definiton and found value
	#[error("could not parse value for port [{0}] into specified type [{1}]")]
	ParsePortValue(ConstString, ConstString),
	/// Port is not defined.
	#[error("Couldn't find port [{0}]")]
	PortError(ConstString),

	/// Something weird happened.
	#[error("unexpected [{0}] in file [{1}] at line [{2}]")]
	Unexpected(ConstString, ConstString, u32),
}
// region:		--- Error
