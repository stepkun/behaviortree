// Copyright © 2025 Stephan Kunz

//! `BehaviorTreeFactory` and `XmlParser` errors.
//!

#[doc(hidden)]
extern crate alloc;

#[doc(hidden)]
#[cfg(feature = "std")]
extern crate std;

// region		--- modules
use crate::ConstString;
use thiserror::Error;
// endregion:	--- modules

// region:		--- Error
/// `factory` error type
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    /// Passthrough for `BehaviorErrors`
    #[error("{0}")]
    Behavior(#[from] crate::behavior::BehaviorError),
    /// Behavior is already registered
    #[error("behavior [{0}] is already registered")]
    BehaviorAlreadyRegistered(ConstString),
    /// Behavior is not registered
    #[error("behavior [{0}] is not registered")]
    BehaviorNotRegistered(ConstString),
    /// A wron BTCPP version is given
    #[error("'BTCPP_format' must be '4'")]
    BtCppFormat,
    /// Children are not allowed for some types of behaviors
    #[error("children are not allowed for behavior category [{0}]")]
    ChildrenNotAllowed(ConstString),
    /// Deadlock situation
    #[error(
        "search for subtree in registry [{0}] caused a deadlock, most probably because this subtree contains himself"
    )]
    DeadLock(ConstString),
    /// Passthrough for libloading Errors
    #[cfg(feature = "std")]
    #[error("{0}")]
    Env(#[from] std::io::Error),
    /// Decorator with more than 1 child
    #[error("the Decorator [{0}] has more than 1 child")]
    DecoratorOnlyOneChild(ConstString),
    /// Decorator with more than 1 child
    #[error("the (Sub)Tree [{0}] has more than 1 child")]
    SubtreeOnlyOneChild(ConstString),
    /// Unsupported XML element:
    #[error("element [{0}] is not supported")]
    ElementNotSupported(ConstString),
    /// Passthrough for libloading Errors
    #[error("{0}")]
    Libloading(#[from] libloading::Error),
    /// Missing a corresponing end tag
    #[error("missing end tag for [{0}]")]
    MissingEndTag(ConstString),
    /// Attribut 'ID' is missing
    #[error("missing attribute 'ID' in tag [{0}]")]
    MissingId(ConstString),
    /// Attribut 'path' is missing
    #[error("missing attribute 'path' in tag [{0}]")]
    MissingPath(ConstString),
    /// Name for a port is not allowed
    #[error("name [{0}] not allowed for a port")]
    NameNotAllowed(ConstString),
    /// The main tree information is missing
    #[error("no 'main_tree_to_execute' name provided")]
    NoMainTreeName,
    /// The main tree information is missing
    #[error("no 'main_tree_to_execute' with name {0} provided")]
    NoMainTree(ConstString),
    /// The main tree information is missing
    #[error("no 'main_tree_to_execute' provided")]
    NoTreeToExecute,
    /// Passthrough for scripting Errors
    #[error("{0}")]
    Scripting(#[from] tinyscript::Error),
    /// Passthrough port error
    #[error("{0}")]
    Port(#[from] crate::port::error::Error),
    /// Port not in defined port list
    #[error("port name [{0}] does not match [{1}]s port list: {2:?}")]
    PortInvalid(ConstString, ConstString, ConstString),
    /// Loading a library failed
    #[error("registering library [{0}] failed with [{0}]")]
    RegisterLib(ConstString, u32),
    /// Subtree already registered
    #[error("subtree with id [{0}] is already registered")]
    SubtreeAlreadyRegistered(ConstString),
    /// The tree is not properly created
    #[error("(sub)tree [{0}] not found in behavior tree")]
    SubtreeNotFound(ConstString),
    /// Passthrough for behavior tree Errors
    #[error("{0}")]
    Tree(#[from] crate::tree::error::Error),
    /// Special attribute values not defined
    #[error("special attribute [{0}] is not supported")]
    UnknownSpecialAttribute(ConstString),
    /// Processing instruction
    #[error("processing instruction [{0}] is not supported")]
    UnsupportedProcessingInstruction(ConstString),
    /// Wrong value for "_autoremap"
    #[error("'_autoremap' must be 'true' or 'false'")]
    WrongAutoremap,
    /// Wrong name for the root element
    #[error("root element must be named 'root'")]
    WrongRootName,
    /// Passthrough for roxmltree Errors
    #[error("{0}")]
    XmlParser(#[from] roxmltree::Error),

    /// A really unexpected error happened
    #[error("unexpected [{0}] in file [{1}] at line [{2}]")]
    Unexpected(ConstString, ConstString, u32),
}
// region:		--- Error
