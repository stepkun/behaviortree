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
	/// The root of the tree s not properly created
	#[error("search for subtree [{0}] caused a deadlock, most probably because this subtree contains himself")]
	DeadLock(ConstString),
	/// The index of a behavior is out of bounds
	#[error("index [{0}] out of bounds")]
	IndexOutOfBounds(usize),
	/// The request type is invalid
	#[error("invalid groot request type [{0}]")]
	InvalidRequestType(u8),
	/// The root of the tree is not properly created
	#[error("root tree [{0}] not found in behavior tree")]
	RootNotFound(ConstString),
	/// The tree is not properly created
	#[error("(sub)tree [{0}] not found in behavior tree")]
	SubtreeNotFound(ConstString),

	/// Something happened that should not have been possible
	#[error("unexpected [{0}] in file [{1}] at line [{2}]")]
	Unexpected(ConstString, ConstString, u32),
}
// region:		--- Error
