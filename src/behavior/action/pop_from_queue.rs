// Copyright © 2025 Stephan Kunz
//! [`PopFromQueue`] [`Action`] implementation.

// region:      --- modules
use alloc::{boxed::Box, string::ToString};
use core::fmt::Debug;
use core::str::FromStr;
use tinyscript::SharedRuntime;

use crate::{
	self as behaviortree, Action,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState, error::BehaviorError, shared_queue::SharedQueue},
	input_port, output_port,
	port::PortList,
	port_list,
	tree::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:		--- globals
/// Port name literals
const POPPED_ITEM: &str = "popped_item";
const QUEUE: &str = "queue";
// endregion:	--- globals

// region:      --- PopFromQueue
/// The [`PopFromQueue`] behavior is used to `pop_front` an element from a [`SharedQueue`].
/// This element is moved into the port `popped_item`.
/// If the queue is empty, the behavior will return Failure.
#[derive(Action, Debug, Default)]
pub struct PopFromQueue<T>
where
	T: Clone + Debug + Default + FromStr + ToString + Send + Sync + 'static,
{
	queue: Option<SharedQueue<T>>,
}

#[async_trait::async_trait]
impl<T> Behavior for PopFromQueue<T>
where
	T: Clone + Debug + Default + FromStr + ToString + Send + Sync,
{
	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		// only on first start
		if self.queue.is_none() {
			// fetch the shared queue
			self.queue = Some(behavior.get::<SharedQueue<T>>(QUEUE)?);
		}
		behavior.set_state(BehaviorState::Running);
		Ok(())
	}

	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		if let Some(queue) = &self.queue {
			if let Some(value) = queue.pop_front() {
				behavior.set::<T>(POPPED_ITEM, value)?;
				Ok(BehaviorState::Success)
			} else {
				Ok(BehaviorState::Failure)
			}
		} else {
			Err(BehaviorError::Composition(
				"PopFromQueue: Queue was not initiialized properly!".into(),
			))
		}
	}

	fn provided_ports() -> PortList {
		port_list![
			input_port!(SharedQueue<T>, QUEUE),
			output_port!(T, POPPED_ITEM),
		]
	}
}
// endregion:   --- PopFromQueue
