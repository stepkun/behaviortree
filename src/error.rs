// Copyright © 2024 Stephan Kunz

//! [`behaviortree`](crate) errors.

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// region		--- modules
use thiserror::Error;
// endregion:	--- modules

// region:		--- types
/// Result type definition for behavior trees.
pub type BehaviorTreeResult<Output = crate::behavior::BehaviorState> = Result<Output, Error>;
// endregion:   --- types

// region:		--- Error
/// `behaviortree` error type
#[derive(Error, Debug)]
pub enum Error {
	/// Pass through from `crate::behavior::BehaviorError`
	#[error("{0}")]
	Behavior(#[from] crate::behavior::error::Error),
	/// Pass through from `crate::blackboard::Error`
	#[error("{0}")]
	Blackboard(#[from] databoard::Error),
	/// Pass through from `crate::factory::Error`
	#[error("{0}")]
	Factory(#[from] crate::factory::error::Error),
	/// Passthrough port error
	#[error("{0}")]
	Port(#[from] crate::port::error::Error),
	#[cfg(feature = "std")]
	/// Pass through from `std::io::Error`
	#[error("{0}")]
	StdIo(#[from] std::io::Error),
	/// Pass through from `crate::tree::Error`
	#[error("{0}")]
	Tree(#[from] crate::tree::error::Error),
	/// Pass through from `woxml::Error`
	#[error("{0}")]
	Woxml(#[from] woxml::Error),
	/// Pass through from `xml::Error`
	#[error("{0}")]
	Xml(#[from] crate::xml::error::Error),
	#[cfg(feature = "std")]
	/// roxmltree Errors
	#[error("{0}")]
	XmlParser(#[from] roxmltree::Error),
	/// roxmltree Errors
	#[cfg(not(feature = "std"))]
	#[error("Error parsing XML")]
	XmlParser,
}
// region:		--- Error
