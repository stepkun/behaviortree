// Copyright © 2025 Stephan Kunz

//! `XmlParser` and `XmlCreator` errors.

#[doc(hidden)]
#[cfg(feature = "std")]
extern crate std;

use crate::ConstString;

/// `xml` error type
#[non_exhaustive]
pub enum Error {
	/// `BtCPP format` is not supported
	BtCppFormat,
	/// The behavior does not allow children
	ChildrenNotAllowed {
		behavior: ConstString,
	},
	/// A wrong pre- or post-condition
	Condition {
		key: ConstString,
		source: crate::BehaviorError,
	},
	/// Pass through errors from databoard
	Databoard {
		key: ConstString,
		source: databoard::Error,
	},
	// A behavior definition cannot be found
	DefinitionNotFound {
		id: ConstString,
	},
	/// Pass through errors from factory/registry
	Factory {
		behavior: ConstString,
		source: crate::factory::error::Error,
	},
	/// A root element at an invalid position
	InvalidRootElement,
	/// Attribute 'ID' is missing
	MissingId {
		tag: ConstString,
	},
	/// Attribute 'path' is missing
	#[cfg(feature = "std")]
	MissingPath {
		tag: ConstString,
	},
	/// The name for the key is not allowed
	NameNotAllowed {
		key: ConstString,
	},
	/// Behavior is not registered
	NotRegistered {
		behavior: ConstString,
	},
	/// The behavior does only allow and must have 1 child
	OneChild {
		behavior: ConstString,
	},
	/// Pass through errors from xml parser
	Parser {
		source: roxmltree::Error,
	},
	/// Port not in defined port list of a behavior
	PortInvalid {
		port: ConstString,
		behavior: ConstString,
	},
	/// Invalid port type
	PortType {
		value: ConstString,
	},
	#[cfg(feature = "std")]
	ReadFile {
		name: ConstString,
		cause: ConstString,
	},
	/// Unknown attribute
	UnknownAttribute {
		key: ConstString,
	},
	/// Unsupported element
	UnsupportedElement {
		tag: ConstString,
	},
	/// Value for auroremap is wrong
	WrongAutoremap,
	/// Name for root element is wrong
	WrongRootName,
}

/// Only a source implementation needed.
impl core::error::Error for Error {
	fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
		match self {
			Self::Condition { key: _, source } => Some(source),
			Self::Databoard { key: _, source } => Some(source),
			#[cfg(feature = "std")]
			Self::Parser { source } => Some(source),
			_ => None,
		}
	}

	// fn cause(&self) -> Option<&dyn core::error::Error> {
	// 	self.source()
	// }

	// fn provide<'a>(&'a self, request: &mut core::error::Request<'a>) {}
}

impl core::fmt::Debug for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::BtCppFormat => write!(f, "BtCppFormat"),
			Self::ChildrenNotAllowed { behavior } => write!(f, "ChildrenNotAllowed(behavior: {behavior})"),
			Self::Condition { key, source } => write!(f, "Condition(key: {key}, err: {source})"),
			Self::Databoard { key, source } => write!(f, "Databoard(key: {key}, err: {source})"),
			Self::DefinitionNotFound { id } => write!(f, "DefinitonNotFound(id: {id})"),
			Self::Factory { behavior, source } => write!(f, "Factory(key: {behavior}, err: {source})"),
			Self::InvalidRootElement => write!(f, "InvalidRootElement"),
			Self::MissingId { tag } => write!(f, "MissingId(tag: {tag})"),
			#[cfg(feature = "std")]
			Self::MissingPath { tag } => write!(f, "MissingPath(tag: {tag})"),
			Self::NameNotAllowed { key } => write!(f, "NameNotAllowed(key: {key})"),
			Self::NotRegistered { behavior } => write!(f, "NotRegistered(behavior: {behavior})"),
			Self::OneChild { behavior } => write!(f, "OneChild(behavior: {behavior})"),
			Self::Parser { source } => write!(f, "Parser(err: {source})"),
			Self::PortInvalid { port, behavior } => {
				write!(f, "PortInvalid(port: {port}, behavior: {behavior})")
			}
			Self::PortType { value } => write!(f, "PortType(value: {value})"),
			#[cfg(feature = "std")]
			Self::ReadFile { name, cause } => write!(f, "ReadFile(name: {name}, cause: {cause}"),
			Self::UnknownAttribute { key } => write!(f, "UnknownAttribute(key: {key})"),
			Self::UnsupportedElement { tag } => write!(f, "UnsupportedElement(tag: {tag})"),
			Self::WrongAutoremap => write!(f, "WrongAutoremap"),
			Self::WrongRootName => write!(f, "WrongRootName"),
		}
	}
}

impl core::fmt::Display for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::BtCppFormat => write!(f, "the attribute 'BTCPP_format' must have the value '4'"),
			Self::ChildrenNotAllowed { behavior } => write!(f, "the leaf behavior {behavior} may not have any children"),
			Self::Condition { key, source } => write!(f, "the pre-/post-condition key: {key} is erronous: {source}"),
			Self::Databoard { key, source } => write!(f, "the databoard key {key} caused the error {source}"),
			Self::DefinitionNotFound { id } => write!(f, "the behavior definition for the id: {id} could not be found"),
			Self::Factory { behavior, source } => write!(f, "registering the behavior {behavior} caused the error {source}"),
			Self::InvalidRootElement => write!(f, "a root element is invalid as child"),
			Self::MissingId { tag } => write!(f, "the tag {tag} is missing an 'ID' attribute"),
			Self::OneChild { behavior } => write!(f, "the behavior {behavior} must have exactly 1 child"),
			#[cfg(feature = "std")]
			Self::MissingPath { tag } => write!(f, "the tag {tag}) is missing a 'path' attribute"),
			Self::NameNotAllowed { key } => write!(f, "the name for the key {key} is not allowed"),
			Self::NotRegistered { behavior } => write!(f, "the behavior {behavior} is not registered"),
			Self::Parser { source } => write!(f, "parsing xml failed with: {source}"),
			Self::PortInvalid { port, behavior } => {
				write!(f, "the port {port}is not in {behavior}'s  portlist")
			}
			Self::PortType { value } => write!(f, "the value {value} is not valis as PortType"),
			#[cfg(feature = "std")]
			Self::ReadFile { name, cause } => write!(f, "file {name} could not be read: {cause}"),
			Self::UnknownAttribute { key } => write!(f, "the attribute with key {key} is unknown"),
			Self::UnsupportedElement { tag } => write!(f, "the element {tag} is not supported"),
			Self::WrongAutoremap => write!(f, "the value for autoremap must be a boolean 'true' or 'false'"),
			Self::WrongRootName => write!(f, "the name for the 'root' element must be 'root'"),
		}
	}
}

impl From<roxmltree::Error> for Error {
	fn from(source: roxmltree::Error) -> Self {
		Self::Parser { source }
	}
}
