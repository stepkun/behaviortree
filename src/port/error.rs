// Copyright © 2025 Stephan Kunz

//! [`behaviortree`](crate) [`Port`](crate::port) errors.

// region		--- modules
use crate::ConstString;
use thiserror::Error;
// endregion:	--- modules

// region:		--- Error
/// `port` error type
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
	/// Passthrough for [`Databoard`](databoard) errors
	#[error("{0}")]
	Blackboard(#[from] databoard::Error),
	/// Could not convert the str into required T
	#[error("could not convert [{0}] into wanted type")]
	CouldNotConvert(ConstString),
	/// Port already in [`PortList`](crate::port::port_list::PortList)
	#[error("name [{0}] already in port list")]
	AlreadyInPortList(ConstString),
	/// Port already in [`Remappings`](crate::blackboard::Remappings)
	#[error("name [{0}] already in remapping list")]
	AlreadyInRemappings(ConstString),
	/// Name for a port is not allowed
	#[error("name [{0}] not allowed for a port")]
	NameNotAllowed(ConstString),
	/// port not found
	#[error("port [{0}] not found")]
	NotFound(ConstString),
}
// region:		--- Error
