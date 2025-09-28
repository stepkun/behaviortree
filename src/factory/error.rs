// Copyright © 2025 Stephan Kunz

//! `BehaviorTreeFactory` and `XmlParser` errors.

#[doc(hidden)]
#[cfg(feature = "std")]
extern crate std;

use crate::ConstString;

/// `factory` error type
#[non_exhaustive]
pub enum Error {
	/// Item is already registered
	AlreadyRegistered {
		/// Name of the item
		name: ConstString,
	},
	/// Tree creation failed
	Create {
		/// Name of the tree to create
		name: ConstString,
		/// The error from xml module
		error: ConstString,
	},
	/// Invalid file path
	#[cfg(feature = "std")]
	InvalidPath {
		/// The given path to file
		path: ConstString,
	},
	#[cfg(feature = "std")]
	/// Pass through errors from libloading
	LibLoading {
		/// Original error
		source: libloading::Error,
	},
	/// Item is not registered
	NotRegistered {
		/// Name of the item
		name: ConstString,
	},
	/// Loading a library failed
	#[cfg(feature = "std")]
	RegisterLib {
		/// Location of the library
		path: ConstString,
		/// Returned result code
		code: u32,
	},
	/// Registration of an XML failed
	RegisterXml {
		/// Name of the tree to create
		name: ConstString,
		/// The error from xml module
		error: ConstString,
	},
	/// Passthrough for scripting Errors
	Scripting {
		/// Original error
		source: tinyscript::Error,
	},
	#[cfg(feature = "std")]
	/// Pass through errors from `std::io`
	StdIo {
		/// Original error
		source: std::io::Error,
	},
}

/// Only a source implementation needed.
impl core::error::Error for Error {
	fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
		match self {
			#[cfg(feature = "std")]
			Self::LibLoading { source } => Some(source),
			Self::Scripting { source } => Some(source),
			#[cfg(feature = "std")]
			Self::StdIo { source } => Some(source),
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
			Self::AlreadyRegistered { name } => write!(f, "AlreadyRegistered(name: {name})"),
			Self::Create { name, error } => write!(f, "Create(name: {name}, error: {error})"),
			#[cfg(feature = "std")]
			Self::InvalidPath { path } => write!(f, "InvalidPath(path: {path})"),
			#[cfg(feature = "std")]
			Self::LibLoading { source } => write!(f, "LibLoading({source})"),
			Self::NotRegistered { name } => write!(f, "NotRegistered(name: {name})"),
			#[cfg(feature = "std")]
			Self::RegisterLib { path, code } => write!(f, "RegisterLib(path: {path}, code: {code})"),
			Self::RegisterXml { name, error } => write!(f, "RegisterXml(name: {name}, error: {error})"),
			Self::Scripting { source } => write!(f, "Scripting({source})"),
			#[cfg(feature = "std")]
			Self::StdIo { source } => write!(f, "StdIo({source})"),
		}
	}
}

impl core::fmt::Display for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::AlreadyRegistered { name } => write!(f, "the item {name} is already registered"),
			Self::Create { name, error } => write!(f, "creation of tree {name} failed with: {error}"),
			#[cfg(feature = "std")]
			Self::InvalidPath { path } => write!(f, "the file path {path} is invalid"),
			#[cfg(feature = "std")]
			Self::LibLoading { source } => write!(f, "accessing library failed with: {source}"),
			Self::NotRegistered { name } => write!(f, "the item {name} is not registered"),
			#[cfg(feature = "std")]
			Self::RegisterLib { path, code } => write!(f, "registration of the library {path} failed with: {code}"),
			Self::RegisterXml { name, error } => write!(f, "registration of XML {name} failed with: {error}"),
			Self::Scripting { source } => write!(f, "accessing scripting failed with: {source}"),
			#[cfg(feature = "std")]
			Self::StdIo { source } => write!(f, "accessing file failed with: {source}"),
		}
	}
}

#[cfg(feature = "std")]
impl From<libloading::Error> for Error {
	fn from(source: libloading::Error) -> Self {
		Self::LibLoading { source }
	}
}

impl From<tinyscript::Error> for Error {
	fn from(source: tinyscript::Error) -> Self {
		Self::Scripting { source }
	}
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
	fn from(source: std::io::Error) -> Self {
		Self::StdIo { source }
	}
}
