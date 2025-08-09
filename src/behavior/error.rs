// Copyright © 2024 Stephan Kunz

//! `behaviortree` behavior errors

#[doc(hidden)]
extern crate alloc;

// region		--- modules
#[cfg(doc)]
use super::BehaviorState;
use crate::ConstString;
use thiserror::Error;
// endregion:	--- modules

// region:		--- BehaviorError
/// `behaviortree` behavior error type
#[derive(Error, Debug)]
pub enum BehaviorError {
    
    /// Pass through from `nanoserde::DeJsonErr`
    #[error("{0}")]
    Nanoserde(#[from] nanoserde::DeJsonErr),
    /// Pass through from `core::num::ParseIntError`
    #[error("{0}")]
    ParseInt(#[from] core::num::ParseIntError),
    /// Pass through blackboard error
    #[error("{0}")]
    Blackboard(#[from] crate::blackboard::error::Error),
    /// Pass through port error
    #[error("{0}")]
    Port(#[from] crate::port::error::Error),
    /// Error in structural composition of a behaviors children
    #[error("{0}")]
    Composition(ConstString),
    /// Pass through float parsing error
    #[error("{0}")]
    FloatParse(#[from] core::num::ParseFloatError),
    /// The index of a behavior is out of bounds
    #[error("index [{0}] out of bounds")]
    IndexOutOfBounds(usize),
    /// Error in internal composition of a behavior
    #[error("{0}")]
    Internal(ConstString),
    /// Pass through join error
    #[error("{0}")]
    JoinError(ConstString),
    /// Attribute is not a post condition
    #[error("attribute [{0}] is not a post condition")]
    NoPostCondition(ConstString),
    /// Attribute is not a pre condition
    #[error("attribute [{0}] is not a pre condition")]
    NoPreCondition(ConstString),
    /// Attribute is not a pre condition
    #[error("tree has no root element")]
    NoRoot,
    /// VM result is not a boolean value
    #[error("result of VM computation is not a boolean value")]
    NotABool,
    /// Variable/Port is not in Blackboard
    #[error("could not find entry [{0}] in blackboard")]
    NotInBlackboard(ConstString),
    /// Parsing error duriong type conversion
    #[error("could not parse value [{0}] in [{1}]")]
    ParseError(ConstString, ConstString),
    /// Type mismatch between port definiton and found value
    #[error("could not parse value for port [{0}] into specified type [{1}]")]
    ParsePortValue(ConstString, ConstString),
    /// Pass through parsing error
    #[error("{0}")]
    Scripting(#[from] tinyscript::Error),
    /// Port has not been defined in behavior
    #[error("port [{0}] is not declared in behavior [{1}]")]
    PortNotDeclared(ConstString, ConstString),
    /// The root of the tree is not properly created
    #[error("tree root [{0}] not found")]
    RootNotFound(ConstString),
    /// An invalid [`BehaviorState`] is reached
    #[error("child node of [{0}] returned state [{1}] when not allowed")]
    State(ConstString, ConstString),
    /// The tree is not properly created
    #[error("(sub)tree [{0}] not found in behavior tree")]
    SubtreeNotFound(ConstString),
    /// Unable to set the post condition
    #[error("unable to set the post condition [{0}]")]
    UnableToSetPostCondition(ConstString),
    /// Unable to set the pre condition
    #[error("unable to set the pre condition [{0}]")]
    UnableToSetPreCondition(ConstString),

    /// Something happened that should not have been possible
    #[error("unexpected [{0}] in file [{1}] at line [{2}]")]
    Unexpected(ConstString, ConstString, u32),
}
// region:		--- BehaviorError
