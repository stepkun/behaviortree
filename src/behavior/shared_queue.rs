// Copyright © 2025 Stephan Kunz
//! [`SharedQueue<T>`]  implementation.

#![allow(dead_code)]

// region:      --- modules
use crate::{self as behaviortree, behavior::error::Error as BehaviorError};
use alloc::collections::vec_deque::VecDeque;
use alloc::string::ToString;
use alloc::sync::Arc;
use behaviortree::Mutex;
use core::fmt::{Debug, Display, Formatter};
use core::str::FromStr;
// endregion:   --- modules

// region:		--- SharedQueue
/// Shared queue implementation for the behaviors
/// - [`Loop<T>`](crate::behavior::decorator::Loop)
/// - [`PopFromQueue<T>`](crate::behavior::action::PopFromQueue)
#[derive(Debug, Default)]
pub struct SharedQueue<T: FromStr + ToString>(pub Arc<Mutex<VecDeque<T>>>);

impl<T> Clone for SharedQueue<T>
where
	T: FromStr + ToString,
{
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<T> Display for SharedQueue<T>
where
	T: FromStr + ToString + Debug,
{
	fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
		write!(f, "{:?}", self.0.lock())
	}
}

impl<T> FromStr for SharedQueue<T>
where
	T: FromStr + ToString,
{
	type Err = behaviortree::behavior::BehaviorError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let queue: Self = Self::with_capacity(s.split(';').count());
		let vals = s.split(';');
		for val in vals {
			let item = match T::from_str(val) {
				Ok(item) => item,
				Err(_err) => {
					return Err(BehaviorError::ParseError {
						value: val.into(),
						src: s.into(),
					});
				}
			};
			queue.push_back(item);
		}
		Ok(queue)
	}
}

impl<T> SharedQueue<T>
where
	T: FromStr + ToString,
{
	/// Create a shared queue with a given starting capacity.
	#[must_use]
	pub fn with_capacity(capacity: usize) -> Self {
		Self(Arc::new(Mutex::new(VecDeque::with_capacity(capacity))))
	}

	/// Removes the last element from the queue and returns it,
	/// or None if it is empty.
	#[must_use]
	pub fn pop_back(&self) -> Option<T> {
		self.0.lock().pop_back()
	}

	/// Removes the first element from the queue and returns it,
	/// or None if it is empty.
	#[must_use]
	pub fn pop_front(&self) -> Option<T> {
		self.0.lock().pop_front()
	}

	/// Appends an element to the back of the queue.
	pub fn push_back(&self, value: T) {
		self.0.lock().push_back(value);
	}

	/// Prepends an element to the queue.
	pub fn push_front(&self, value: T) {
		self.0.lock().push_front(value);
	}
}
// endregion:	--- SharedQueue
