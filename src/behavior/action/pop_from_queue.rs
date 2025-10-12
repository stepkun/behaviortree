// Copyright © 2025 Stephan Kunz
//! [`PopFromQueue`] [`Action`] implementation.

// region:      --- modules
use crate::{
	self as behaviortree, Action,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState, shared_queue::SharedQueue},
	input_port, output_port,
	port::PortList,
	port_list,
	tree::BehaviorTreeElementList,
};
use alloc::{boxed::Box, string::ToString};
use core::fmt::Debug;
use core::str::FromStr;
use tinyscript::SharedRuntime;
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
///
/// There are the predefined behaviors
/// - PopBool: gated behind feature `pop_bool`
/// - PopDouple: gated behind feature `pop_double`
/// - PopInt: gated behind feature `pop_int` (int is i32)
/// - PopString: gated behind feature `pop_string`
///
/// The raw version is gated behind feature `pop_from_queue`.
#[derive(Action, Debug, Default)]
pub struct PopFromQueue<T>
where
	T: Clone + Debug + Default + FromStr + ToString + Send + Sync + 'static,
{
	/// A temporary queue to store fixed queue definitions
	tmp_queue: Option<SharedQueue<T>>,
}

#[async_trait::async_trait]
impl<T> Behavior for PopFromQueue<T>
where
	T: Clone + Debug + Default + FromStr + ToString + Send + Sync,
{
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		// get a value
		let value = if let Some(const_queue) = &self.tmp_queue {
			const_queue.pop_front()
		} else {
			match behavior.get_mut_ref::<SharedQueue<T>>(QUEUE) {
				Ok(q) => q.pop_front(),
				#[allow(clippy::collapsible_match)]
				Err(err) => match &err {
					crate::port::error::Error::Databoard { source } => match source {
						databoard::Error::Assignment { key: _, value } => {
							let q = SharedQueue::from_str(value)?;
							let first = q.pop_front();
							self.tmp_queue = Some(q);
							first
						}
						_ => return Err(err.into()),
					},
					_ => return Err(err.into()),
				},
			}
		};

		if let Some(value) = value {
			behavior.set::<T>(POPPED_ITEM, value)?;
			Ok(BehaviorState::Success)
		} else {
			self.tmp_queue = None;
			Ok(BehaviorState::Failure)
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
