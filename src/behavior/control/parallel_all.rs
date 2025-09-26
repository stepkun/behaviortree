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
	tree::BehaviorTreeElementList,
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
#[derive(Control, Debug, Default)]
pub struct ParallelAll {
	/// The amount of completed sub behaviors that failed.
	failure_count: i32,
	/// The list of completed sub behaviors
	completed_list: BTreeSet<usize>,
}

#[async_trait::async_trait]
impl Behavior for ParallelAll {
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		self.failure_count = 0;
		self.completed_list.clear();
		Ok(())
	}

	#[allow(clippy::cast_possible_truncation)]
	#[allow(clippy::cast_possible_wrap)]
	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		// check composition only once at start
		// The maximum allowed failures.
		// "-1" signals any number.
		let failure_threshold = behavior.get(MAX_FAILURES).unwrap_or(-1);
		self.failure_count = 0;

		if (children.len() as i32) < failure_threshold {
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
		behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		let failure_threshold = behavior.get(MAX_FAILURES).unwrap_or(-1);
		let children_count = children.len();

		let mut skipped_count = 0;

		for i in 0..children_count {
			// Skip completed behaviors
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
			let state = if (failure_threshold >= 0) && (self.failure_count > failure_threshold) {
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
