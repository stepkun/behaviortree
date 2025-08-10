// Copyright © 2025 Stephan Kunz

//! `behaviortree` Port errors

#[doc(hidden)]
extern crate alloc;

// region		--- modules
#[cfg(doc)]
use super::{PortList, PortRemappings};
use crate::ConstString;
use thiserror::Error;
// endregion:	--- modules

// region:		--- Error
/// `port` error type
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    /// Passthrough for Blackboard `Errors`
    #[error("{0}")]
    Blackboard(#[from] crate::blackboard::error::Error),
    /// Could not convert the str into required T
    #[error("could not convert [{0}] into wanted type")]
    CouldNotConvert(ConstString),
    /// Port already in [`PortList`]
    #[error("name [{0}] already in port list")]
    AlreadyInPortList(ConstString),
    /// Port already in [`PortRemappings`]
    #[error("name [{0}] already in remapping list")]
    AlreadyInRemappings(ConstString),
    /// Name for a port is not allowed
    #[error("name [{0}] not allowed for a port")]
    NameNotAllowed(ConstString),
}
// region:		--- Error
