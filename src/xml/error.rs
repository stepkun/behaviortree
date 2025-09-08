// Copyright © 2025 Stephan Kunz

//! `XmlParser` and `XmlCreator` errors.

#[doc(hidden)]
#[cfg(feature = "std")]
extern crate std;

// region		--- modules
use crate::ConstString;
use thiserror::Error;
// endregion:	--- modules

// region:		--- Error
/// `xml` error type
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
	/// Passthrough for `std::io::Error`s
	#[cfg(feature = "std")]
	#[error("{0}")]
	Env(#[from] std::io::Error),
	/// Passthrough for roxmltree Errors
	#[cfg(feature = "std")]
	#[error("{0}")]
	XmlParser(#[from] roxmltree::Error),
	/// roxmltree Errors
	#[cfg(not(feature = "std"))]
	#[error("Error parsing XML")]
	XmlParser,
	/// Behavior is not registered
	#[error("behavior [{0}] is not registered")]
	BehaviorNotRegistered(ConstString),
	/// A wrong BTCPP version is given
	#[error("'BTCPP_format' must be '4'")]
	BtCppFormat,
	/// Children are not allowed for some categories of behaviors
	#[error("children are not allowed for behavior category [{0}]")]
	ChildrenNotAllowed(ConstString),
	/// Decorator with more than 1 child
	#[error("the Decorator [{0}] has more than 1 child")]
	DecoratorOneChild(ConstString),
	/// Unsupported XML element:
	#[error("element [{0}] is not supported")]
	ElementNotSupported(ConstString),
	/// Invalid root element below a root
	#[error("a root element below a root element is not allowed")]
	InvalidRootElement,
	/// Attribut 'ID' is missing
	#[error("missing attribute 'ID' in tag [{0}]")]
	MissingId(ConstString),
	/// Attribut 'path' is missing
	#[error("missing attribute 'path' in tag [{0}]")]
	MissingPath(ConstString),
	/// Name for a port is not allowed
	#[error("name [{0}] not allowed for a port")]
	NameNotAllowed(ConstString),
	/// Port not in defined port list
	#[error("port name [{0}] does not match [{1}]s port list: {2:?}")]
	PortInvalid(ConstString, ConstString, ConstString),
	/// Postcondition error
	#[error("add postcondition for [{0}] failed due to [{1}]")]
	Postcondition(ConstString, crate::BehaviorError),
	/// Precondition error
	#[error("add precondition for [{0}] failed due to [{1}]")]
	Precondition(ConstString, crate::BehaviorError),
	/// Read of file failed
	#[cfg(feature = "std")]
	#[error("file [{0}] could not be read: {1}")]
	ReadFile(ConstString, ConstString),
	/// Registration error
	#[error("registration of [{0}] failed due to [{1}]")]
	Registration(ConstString, crate::factory::error::Error),
	/// Remapping error
	#[error("add remapping failed due to [{0}]")]
	Remapping(crate::blackboard::error::Error),
	/// The subtree is not registered
	#[error("(sub)tree [{0}] not found in registry")]
	SubtreeNotFound(ConstString),
	/// Subtree with more than 1 child
	#[error("the (Sub)Tree [{0}] has more than 1 child")]
	SubtreeOneChild(ConstString),
	/// Special attribute values not defined
	#[error("special attribute [{0}] is not supported")]
	UnknownSpecialAttribute(ConstString),
	/// Unsupported processing instruction
	#[error("processing instruction [{0}] is not supported")]
	UnsupportedProcessingInstruction(ConstString),
	/// Wrong value for "_autoremap"
	#[error("'_autoremap' must be 'true' or 'false'")]
	WrongAutoremap,
	/// Wrong name for the root element
	#[error("root element must be named 'root'")]
	WrongRootName,
}
// region:		--- Error
