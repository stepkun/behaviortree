// Copyright © 2025 Stephan Kunz
//! [`ParallelAll`] [`Control`] implementation.

// region:      --- modules
use alloc::boxed::Box;
use alloc::collections::btree_set::BTreeSet;
use tinyscript::SharedRuntime;

use crate::{
	self as behaviortree, Control, IDLE,
	behavior::{Behavior, BehaviorData, BehaviorResult, BehaviorState, error::BehaviorError},
	input_port,
	port::PortList,
	port_list,
	tree::tree_element_list::ConstBehaviorTreeElementList,
};
// endregion:   --- modules

// region:		--- globals
/// Port name literals
const MAX_FAILURES: &str = "max_failures";
// endregion:	--- globals

// region:      --- ParallelAll
/// A [`ParallelAll`] executes its children __concurrently__ in one thread.
///
/// In difference to the [`Parallel`](crate::behavior::control::parallel::Parallel) behavior,
/// the [`ParallelAll`] finishes the execution of all its children before deciding whether its a Success or a Failure.
#[derive(Control, Debug)]
pub struct ParallelAll {
	/// The maximum allowed failures.
	/// "-1" signals any number.
	failure_threshold: i32,
	/// The amount of completed sub behaviors that failed.
	failure_count: i32,
	/// The list of completed sub behaviors
	completed_list: BTreeSet<usize>,
}

impl Default for ParallelAll {
	fn default() -> Self {
		Self {
			failure_threshold: -1,
			failure_count: 0,
			completed_list: BTreeSet::default(),
		}
	}
}

#[async_trait::async_trait]
impl Behavior for ParallelAll {
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		self.failure_threshold = -1;
		self.completed_list.clear();
		self.failure_count = 0;
		Ok(())
	}

	#[allow(clippy::cast_possible_truncation)]
	#[allow(clippy::cast_possible_wrap)]
	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		// check composition only once at start
		self.failure_threshold = behavior.get(MAX_FAILURES).unwrap_or(-1);

		if (children.len() as i32) < self.failure_threshold {
			return Err(BehaviorError::Composition(
				"Number of children is less than the threshold. Can never fail.".into(),
			));
		}
		behavior.set_state(BehaviorState::Running);
		Ok(())
	}

	#[allow(clippy::cast_possible_truncation)]
	#[allow(clippy::cast_possible_wrap)]
	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		children: &mut ConstBehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		let children_count = children.len();

		let mut skipped_count = 0;

		for i in 0..children_count {
			// Skip completed node
			if self.completed_list.contains(&i) {
				continue;
			}

			let state = children[i].tick(runtime).await?;
			match state {
				BehaviorState::Success => {
					self.completed_list.insert(i);
				}
				BehaviorState::Failure => {
					self.completed_list.insert(i);
					self.failure_count += 1;
				}
				BehaviorState::Skipped => skipped_count += 1,
				BehaviorState::Running => {}
				// Throw error, should never happen
				BehaviorState::Idle => {
					return Err(BehaviorError::State("ParallelAll".into(), IDLE.into()));
				}
			}
		}

		if skipped_count == children_count {
			return Ok(BehaviorState::Skipped);
		}

		let sum = skipped_count + self.completed_list.len();
		if sum >= children_count {
			let state = if (self.failure_threshold >= 0) && (self.failure_threshold <= self.failure_count) {
				BehaviorState::Failure
			} else {
				BehaviorState::Success
			};

			// Done!
			children.halt(runtime)?;
			self.completed_list.clear();

			return Ok(state);
		}

		Ok(BehaviorState::Running)
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(i32, MAX_FAILURES)]
	}
}
// endregion:   --- ParallelAll
