// Copyright © 2025 Stephan Kunz

//! `BehaviorTreeFactory` and `XmlParser` errors.

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
	// /// Passthrough for `BehaviorErrors`
	// #[error("{0}")]
	// Behavior(#[from] crate::behavior::BehaviorError),
	// /// Passthrough port error
	// #[error("{0}")]
	// Port(#[from] crate::port::error::Error),
	// /// Passthrough for behavior tree Errors
	// #[error("{0}")]
	// Tree(#[from] crate::tree::error::Error),
	/// Passthrough for `libloading::Error`s
	// #[cfg(feature = "std")]
	// #[error("{0}")]
	// XmlParser(#[from] roxmltree::Error),
	// /// roxmltree Errors
	// #[cfg(not(feature = "std"))]
	// #[error("Error parsing XML")]
	// XmlParser,
	/// Passthrough for `std::io::Error`s
	#[cfg(feature = "std")]
	#[error("{0}")]
	Env(#[from] std::io::Error),
	#[cfg(feature = "std")]
	/// Passthrough for libloading Errors
	#[error("{0}")]
	Libloading(#[from] libloading::Error),
	/// Passthrough for scripting Errors
	#[error("{0}")]
	Scripting(#[from] tinyscript::Error),
	/// Behavior is already registered
	#[error("behavior [{0}] is already registered")]
	BehaviorAlreadyRegistered(ConstString),
	/// Behavior is not registered
	#[error("behavior [{0}] is not registered")]
	BehaviorNotRegistered(ConstString),
	/// Creation of tree failed
	#[error("creation of (sub)tree [{0}] failed")]
	Create(ConstString),
	/// Deadlock situation
	#[error("search for subtree in registry [{0}] caused a deadlock, most probably because this subtree contains himself")]
	DeadLock(ConstString),
	/// Missing a corresponing end tag
	#[error("missing end tag for [{0}]")]
	MissingEndTag(ConstString),
	/// The main tree information is missing
	#[error("no 'main_tree_to_execute' name provided")]
	NoMainTreeName,
	/// The main tree information is missing
	#[error("no 'main_tree_to_execute' with name {0} provided")]
	NoMainTree(ConstString),
	/// The main tree information is missing
	#[error("no 'main_tree_to_execute' provided")]
	NoTreeToExecute,
	/// Register XML failed
	#[error("registering xml failed")]
	Register,
	/// Loading a library failed
	#[error("registering library [{0}] failed with [{1}]")]
	RegisterLib(ConstString, u32),
	/// Subtree already registered
	#[error("subtree with id [{0}] is already registered")]
	SubtreeAlreadyRegistered(ConstString),

	/// A really unexpected error happened
	#[error("unexpected [{0}] in file [{1}] at line [{2}]")]
	Unexpected(ConstString, ConstString, u32),
}
// region:		--- Error
