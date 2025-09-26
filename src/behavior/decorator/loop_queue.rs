// Copyright © 2025 Stephan Kunz
//! [`Loop<T>`] [`Decorator`] implementation.

// region:      --- modules
use alloc::{boxed::Box, string::ToString};
use core::fmt::Debug;
use core::str::FromStr;
use tinyscript::SharedRuntime;

use crate::{
	self as behaviortree, Decorator,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState, shared_queue::SharedQueue},
	inout_port, input_port, output_port,
	port::PortList,
	port_list,
	tree::BehaviorTreeElementList,
};
// endregion:   --- modules

// region:		--- globals
/// Port name literals
const IF_EMPTY: &str = "if_empty";
const QUEUE: &str = "queue";
const VALUE: &str = "value";
// endregion:	--- globals

// region:      --- Loop
/// The [`Loop`] behavior is used to `pop_front` elements from a [`SharedQueue`].
/// This element is copied into the port `value` and the child will be executed
/// as long as there are elements in the queue.
#[derive(Decorator, Debug, Default)]
pub struct Loop<T>
where
	T: Clone + Debug + Default + FromStr + ToString + Send + Sync + 'static,
{
	/// A temporary queue to store fixed queue definitions
	tmp_queue: Option<SharedQueue<T>>,
}

#[async_trait::async_trait]
impl<T> Behavior for Loop<T>
where
	T: Clone + Debug + Default + FromStr + ToString + Send + Sync,
{
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		async fn inner_tick(children: &mut BehaviorTreeElementList, runtime: &SharedRuntime) -> BehaviorResult {
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

		behavior.set_state(BehaviorState::Running);

		// get a value
		let value = if let Some(const_queue) = &self.tmp_queue {
			const_queue.pop_front()
		} else {
			match behavior.get_mut_ref::<SharedQueue<T>>(QUEUE) {
				Ok(q) => q.pop_front(),
				Err(err) => match err {
					crate::port::error::Error::Blackboard(error) => match error {
						databoard::Error::Assignment { key: _, value } => {
							let q = SharedQueue::from_str(&value)?;
							let first = q.pop_front();
							self.tmp_queue = Some(q);
							first
						}
						_ => return Err(error.into()),
					},
					_ => return Err(err.into()),
				},
			}
		};

		if let Some(value) = value {
			behavior.set::<T>(VALUE, value)?;
			inner_tick(children, runtime).await
		} else {
			self.tmp_queue = None;
			let state = behavior.get::<BehaviorState>(IF_EMPTY)?;
			Ok(state)
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
			output_port!(T, VALUE),
		]
	}
}
// endregion:   --- Loop
