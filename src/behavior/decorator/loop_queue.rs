// Copyright © 2025 Stephan Kunz
//! `Loop` behavior implementation

// region:      --- modules
use alloc::{boxed::Box, string::ToString};
use core::fmt::Debug;
use core::str::FromStr;
use tinyscript::SharedRuntime;

use crate as behaviortree;
use crate::behavior::shared_queue::SharedQueue;
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
				return Err(BehaviorError::Composition("Loop<T> must have a single child!".into()));
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
				behavior.set::<T>(VALUE, value)?;
				inner_tick(children, runtime).await
			} else {
				Ok(self.state)
			}
		} else {
			Err(BehaviorError::Composition(
				"Loop<T> : Queue was not initiialized properly!".into(),
			))
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
