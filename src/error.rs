// Copyright © 2024 Stephan Kunz

//! [`behaviortree`](crate) errors.

#[doc(hidden)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::boxed::Box;

// region:		--- types
/// Result type definition for behavior trees.
pub type BehaviorTreeResult<Output = crate::behavior::BehaviorState> = Result<Output, Error>;
// endregion:   --- types

/// `factory` error type
#[non_exhaustive]
pub struct Error {
	/// Module name
	module: &'static str,
	/// Original error
	source: Box<dyn core::error::Error>,
}

/// Only a source implementation needed.
impl core::error::Error for Error {
	fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
		Some(self.source.as_ref())
	}

	// fn cause(&self) -> Option<&dyn core::error::Error> {
	// 	self.source()
	// }

	// fn provide<'a>(&'a self, request: &mut core::error::Request<'a>) {}
}

impl core::fmt::Debug for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "Behaviortree(module: {}, error: {}", self.module, self.source)
	}
}

impl core::fmt::Display for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "Behaviortree module '{}' failed with: {}", self.module, self.source)
	}
}

impl From<crate::behavior::error::Error> for Error {
	fn from(source: crate::behavior::error::Error) -> Self {
		Self {
			module: "behavior",
			source: Box::new(source),
		}
	}
}

impl From<crate::factory::error::Error> for Error {
	fn from(source: crate::factory::error::Error) -> Self {
		Self {
			module: "factory",
			source: Box::new(source),
		}
	}
}

impl From<crate::tree::error::Error> for Error {
	fn from(source: crate::tree::error::Error) -> Self {
		Self {
			module: "tree",
			source: Box::new(source),
		}
	}
}

impl From<databoard::Error> for Error {
	fn from(source: databoard::Error) -> Self {
		Self {
			module: "databoard",
			source: Box::new(source),
		}
	}
}

impl From<woxml::Error> for Error {
	fn from(source: woxml::Error) -> Self {
		Self {
			module: "woxml",
			source: Box::new(source),
		}
	}
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
	fn from(source: std::io::Error) -> Self {
		Self {
			module: "std::io",
			source: Box::new(source),
		}
	}
}
