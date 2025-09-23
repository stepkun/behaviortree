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
	#[cfg(feature = "std")]
	#[error("creation of (sub)tree [{0}] failed: {1}")]
	Create(ConstString, ConstString),
	/// Creation of tree failed
	#[cfg(not(feature = "std"))]
	#[error("creation of (sub)tree [{0}] failed")]
	Create(ConstString),
	/// `TreeNodesModel` entry already registered
	#[error("a 'TreeNodesModel' registration with [{0}] already exists")]
	EntryAlreadyRegistered(ConstString),
	/// Invalid plugin path
	#[cfg(feature = "std")]
	#[error("invalid plugin path [{path}]")]
	InvalidPath {
		/// The given path to plugin
		path: ConstString,
	},
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
	#[cfg(feature = "std")]
	#[error("registering xml failed due to {0}")]
	RegisterXml(ConstString),
	/// Register XML failed
	#[cfg(not(feature = "std"))]
	#[error("registering xml failed")]
	RegisterXml,
	/// Loading a library failed
	#[error("registering library [{0}] failed with [{1}]")]
	RegisterLib(ConstString, u32),
	/// Subtree already registered
	#[error("subtree with id [{0}] is already registered")]
	SubtreeAlreadyRegistered(ConstString),
}
// region:		--- Error
