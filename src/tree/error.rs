// Copyright © 2025 Stephan Kunz

//! [`BehaviorTree`](crate::tree) tree errors.

#[doc(hidden)]
extern crate alloc;

// region		--- modules
use crate::ConstString;
use thiserror::Error;
// endregion:	--- modules

// region:		--- Error
/// `tree` error type
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
	/// Pass through behavior error
	#[error("{0}")]
	Behavior(#[from] crate::behavior::error::BehaviorError),
	/// The index of a behavior is out of bounds
	#[error("index [{0}] out of bounds")]
	IndexOutOfBounds(usize),
	/// The request type is invalid
	#[error("invalid groot request type [{0}]")]
	InvalidRequestType(u8),
	/// The tree depth limit is exceeded
	#[error("recursion limit exceeded in tree element [{0}]")]
	RecursionLimit(ConstString),
	/// The root of the tree is not properly created
	#[error("root tree [{0}] not found in behavior tree")]
	RootNotFound(ConstString),
	/// The tree is not properly created
	#[error("(sub)tree [{0}] not found in behavior tree")]
	SubtreeNotFound(ConstString),
}
// region:		--- Error
