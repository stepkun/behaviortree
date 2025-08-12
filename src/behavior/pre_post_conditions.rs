// Copyright © 2025 Stephan Kunz

//! `PreConditions` and `PostConditions`.
//!

// region		--- modules
use crate::ConstString;
use core::ops::{Deref, DerefMut};

use super::error::BehaviorError;
// endregion:	--- modules

// region:      --- Conditions
/// Helper struct to reduce amount of parameters
pub struct Conditions {
	pub(crate) pre: PreConditions,
	pub(crate) post: PostConditions,
}
// endregion:	--- Conditions

// region:      --- PreConditions
/// Names and order of the `PreConditions`.
pub const PRE_CONDITIONS: [&str; 4] = ["_failureif", "_successif", "_skipif", "_while"];

/// Array holding the pre conditions.
#[derive(Default)]
pub struct PreConditions(pub(crate) Option<[Option<ConstString>; PRE_CONDITIONS.len()]>);

impl Deref for PreConditions {
	type Target = Option<[Option<ConstString>; PRE_CONDITIONS.len()]>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for PreConditions {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl PreConditions {
	/// Get a pre condition.
	#[must_use]
	pub fn get(&self, name: &str) -> Option<&ConstString> {
		if self.0.is_some() {
			let op = (0..PRE_CONDITIONS.len()).find(|&i| PRE_CONDITIONS[i] == name);
			op.and_then(|index| {
				self.0
					.as_ref()
					.map_or_else(|| None, |array| array[index].as_ref())
			})
		} else {
			None
		}
	}

	/// Set a pre condition.
	/// # Errors
	/// - if name is not a pre condition
	pub fn set(&mut self, name: &str, script: &str) -> Result<(), BehaviorError> {
		// lazy init
		if self.0.is_none() {
			self.0 = Some([None, None, None, None]);
		}

		let op = (0..PRE_CONDITIONS.len()).find(|&i| PRE_CONDITIONS[i] == name);
		if let Some(index) = op {
			self.0.as_mut().map_or_else(
				|| Err(BehaviorError::UnableToSetPreCondition(name.into())),
				|array| {
					array[index] = Some(script.into());
					Ok(())
				},
			)
		} else {
			Err(BehaviorError::NoPreCondition(name.into()))
		}
	}
}
// endregion:   --- PreConditions

// region:      --- PostConditions
/// Names and order of the `PostConditions`.
pub const POST_CONDITIONS: [&str; 4] = ["_onHalted", "_onFailure", "_onSuccess", "_post"];

/// Array holding the post conditions.
#[derive(Default)]
pub struct PostConditions(pub(crate) Option<[Option<ConstString>; POST_CONDITIONS.len()]>);

impl Deref for PostConditions {
	type Target = Option<[Option<ConstString>; POST_CONDITIONS.len()]>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for PostConditions {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl PostConditions {
	/// Get a post condition.
	#[must_use]
	pub fn get(&self, name: &str) -> Option<&ConstString> {
		if self.0.is_some() {
			let op = (0..POST_CONDITIONS.len()).find(|&i| POST_CONDITIONS[i] == name);
			op.and_then(|index| {
				self.0
					.as_ref()
					.map_or_else(|| None, |array| array[index].as_ref())
			})
		} else {
			None
		}
	}

	/// Set a post condition.
	/// # Errors
	/// - if name is not a post condition
	pub fn set(&mut self, name: &str, script: &str) -> Result<(), BehaviorError> {
		// lazy init
		if self.0.is_none() {
			self.0 = Some([None, None, None, None]);
		}

		let op = (0..POST_CONDITIONS.len()).find(|&i| POST_CONDITIONS[i] == name);
		if let Some(index) = op {
			self.0.as_mut().map_or_else(
				|| Err(BehaviorError::UnableToSetPostCondition(name.into())),
				|array| {
					array[index] = Some(script.into());
					Ok(())
				},
			)
		} else {
			Err(BehaviorError::NoPostCondition(name.into()))
		}
	}
}
// endregion:   --- PostConditions
