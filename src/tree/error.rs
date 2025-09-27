// Copyright © 2025 Stephan Kunz

//! [`BehaviorTree`](crate::tree) tree errors.

#[doc(hidden)]
extern crate alloc;

use crate::ConstString;

/// Tree errors
#[non_exhaustive]
pub enum Error {
	/// element with index is not found
	SubtreeNotFound {
		/// The affected index
		index: usize,
	},
	/// Invalid Groot request type
	InvalidRequestType {
		/// The request type value
		value: u8,
	},
	/// Recursion limit  of 127 is reached
	RecursionLimit {
		/// The affected behavior
		behavior: ConstString,
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
			// Self::Behavior { source } => write!(f, "Behavior(source: {source}"),
			Self::SubtreeNotFound { index } => write!(f, "IndexNotFound({index})"),
			Self::InvalidRequestType { value } => write!(f, "InvalidRequestType({value})"),
			Self::RecursionLimit { behavior } => write!(f, "RecursionLimit({behavior})"),
		}
	}
}

impl core::fmt::Display for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			// Self::Behavior { source } => write!(f, "Behavior(source: {source}"),
			Self::SubtreeNotFound { index } => write!(f, "the subtree with the index {index} cannot be found"),
			Self::InvalidRequestType { value } => write!(f, "an invalid request type {value} was sent from Groot2"),
			Self::RecursionLimit { behavior } => write!(f, "recursion limit of '127' is reached for behavior {behavior}"),
		}
	}
}
