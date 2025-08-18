// Copyright © 2025 Stephan Kunz

//! `Loop` behavior implementation
//!

// region:      --- modules
use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use alloc::{boxed::Box, string::ToString};
use core::fmt::{Debug, Display, Formatter};
use core::str::FromStr;
use spin::Mutex;
use tinyscript::SharedRuntime;

use crate as behaviortree;
use crate::{
	Decorator,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState, error::BehaviorError},
	inout_port, input_port, output_port,
	port::PortList,
	port_list,
	tree::tree_element_list::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:		--- globals
/// Port name literals
const IF_EMPTY: &str = "if_empty";
const QUEUE: &str = "queue";
// endregion:	--- globals

// region:		--- SharedQueue
/// Shared queue implementation for the [`Loop`] behavior
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
				Err(_err) => return Err(BehaviorError::ParseError(val.into(), s.into())),
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

	// /// Removes the last element from the queue and returns it,
	// /// or None if it is empty.
	// #[must_use]
	// pub fn pop_back(&self) -> Option<T> {
	//     self.0.lock().pop_back()
	// }

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

	// /// Prepends an element to the queue.
	// pub fn push_front(&self, value: T) {
	//     self.0.lock().push_front(value);
	// }
}
// endregion:	--- SharedQueue

// region:      --- Loop
/// The [`Loop`] behavior is used to `pop_front` elements from a [`VecDeque`].
/// This element is copied into the port "value" and the child will be executed
/// as long as there are elements in the queue.
#[derive(Decorator, Debug, Default)]
pub struct Loop<T>
where
	T: Clone + Debug + Default + FromStr + ToString + Send + Sync + 'static,
{
	queue: Option<SharedQueue<T>>,
	state: BehaviorState,
}

#[async_trait::async_trait]
impl<T> Behavior for Loop<T>
where
	T: Clone + Debug + Default + FromStr + ToString + Send + Sync,
{
	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		// only on first start
		if self.queue.is_none() {
			// check composition
			if children.len() != 1 {
				return Err(BehaviorError::Composition("Loop must have a single child!".into()));
			}
			// fetch if_empty value
			self.state = behavior.get::<BehaviorState>(IF_EMPTY)?;
			// fetch the shared queue
			self.queue = Some(behavior.get::<SharedQueue<T>>(QUEUE)?);
		}
		behavior.set_state(BehaviorState::Running);
		Ok(())
	}

	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut ConstBehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		async fn inner_tick(children: &mut ConstBehaviorTreeElementList, runtime: &SharedRuntime) -> BehaviorResult {
			let child_state = children[0].tick(runtime).await?;
			if child_state.is_completed() {
				children[0].halt_children(runtime)?;
			}
			if child_state == BehaviorState::Failure {
				Ok(BehaviorState::Failure)
			} else {
				Ok(BehaviorState::Running)
			}
		}

		if let Some(queue) = &self.queue {
			if let Some(value) = queue.pop_front() {
				behavior.set::<T>("value", value)?;
				inner_tick(children, runtime).await
			} else {
				Ok(self.state)
			}
		} else {
			Err(BehaviorError::Composition("Queue was not initiialized properly!".into()))
		}
	}

	fn provided_ports() -> PortList {
		port_list![
			inout_port!(SharedQueue<T>, QUEUE),
			input_port!(
				BehaviorState,
				IF_EMPTY,
				BehaviorState::Success,
				"State to return if queue is empty: SUCCESS, FAILURE, SKIPPED"
			),
			output_port!(T, "value"),
		]
	}
}
// endregion:   --- Loop
