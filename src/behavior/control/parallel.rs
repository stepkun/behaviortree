// Copyright © 2025 Stephan Kunz
//! [`Parallel`] [`Control`] implementation.

// region:      --- modules
use crate::{
	self as behaviortree, Control,
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorResult, BehaviorState},
	input_port,
	port::PortList,
	port_list,
	tree::BehaviorTreeElementList,
};
use alloc::boxed::Box;
use alloc::collections::btree_set::BTreeSet;
use tinyscript::SharedRuntime;
// endregion:   --- modules

// region:      --- Parallel
/// A [`Parallel`] executes its children __concurrently__ in one thread.
///
/// The behavior is completed either when the `success_threshold` or the `failure_threshold` is reached.
/// These are configured using the ports `success_count` and `failure_count`.
/// If any of the thresholds is reached, still running children will be halted.
/// This differs from the [`ParallelAll`](crate::behavior::control::parallel_all::ParallelAll) behavior.
#[derive(Control, Debug, Default)]
pub struct Parallel {
	/// The amount of completed sub behaviors that succeeded.
	success_count: i32,
	/// The amount of completed sub behaviors that failed.
	failure_count: i32,
	/// The list of completed sub behaviors
	completed_list: BTreeSet<usize>,
}

/// The port names
const SUCCESS_COUNT: &str = "success_count";
const FAILURE_COUNT: &str = "failure_count";

#[async_trait::async_trait]
impl Behavior for Parallel {
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		self.completed_list.clear();
		self.success_count = 0;
		self.failure_count = 0;
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
		// The minimum needed Successes to return a Success.
		// "-1" signals any number.
		let success_threshold = behavior.get(SUCCESS_COUNT).unwrap_or(-1);
		// The maximum allowed failures.
		// "-1" signals any number.
		let failure_threshold = behavior.get(FAILURE_COUNT).unwrap_or(-1);

		let children_count = children.len();

		if (children_count as i32) < success_threshold {
			return Err(BehaviorError::Composition {
				txt: "Number of children is less than the threshold. Can never succeed.".into(),
			});
		}

		if (children_count as i32) < failure_threshold {
			return Err(BehaviorError::Composition {
				txt: "Number of children is less than the threshold. Can never fail.".into(),
			});
		}
		behavior.set_state(BehaviorState::Running);
		Ok(())
	}

	#[allow(clippy::cast_possible_truncation)]
	#[allow(clippy::cast_possible_wrap)]
	#[allow(clippy::set_contains_or_insert)]
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		children: &mut BehaviorTreeElementList,
		runtime: &SharedRuntime,
	) -> BehaviorResult {
		// The minimum needed Successes to return a Success.
		// "-1" signals any number.
		let success_threshold = behavior.get(SUCCESS_COUNT).unwrap_or(-1);
		// The maximum allowed failures.
		// "-1" signals any number.
		let failure_threshold = behavior.get(FAILURE_COUNT).unwrap_or(-1);
		let children_count = children.len();

		let mut skipped_count = 0;

		for i in 0..children_count {
			// Skip completed node
			if !self.completed_list.contains(&i) {
				let child = &mut children[i];
				match child.tick(runtime).await? {
					BehaviorState::Skipped => skipped_count += 1,
					BehaviorState::Success => {
						self.completed_list.insert(i);
						self.success_count += 1;
					}
					BehaviorState::Failure => {
						self.completed_list.insert(i);
						self.failure_count += 1;
					}
					BehaviorState::Running => {}
					// Throw error, should never happen
					BehaviorState::Idle => {
						return Err(BehaviorError::State {
							behavior: "Parallel".into(),
							state: BehaviorState::Idle,
						});
					}
				}
			}

			let sum = self.failure_count + self.success_count + skipped_count;
			if sum >= children_count as i32 {
				let state = if skipped_count == children_count as i32 {
					BehaviorState::Skipped
				} else if failure_threshold <= 0 && success_threshold <= 0 {
					BehaviorState::Success
				} else if failure_threshold <= 0 {
					if self.success_count >= success_threshold {
						BehaviorState::Success
					} else {
						BehaviorState::Failure
					}
				} else if (self.failure_count > failure_threshold) || (self.success_count < success_threshold) {
					BehaviorState::Failure
				} else {
					BehaviorState::Success
				};

				self.completed_list.clear();
				self.success_count = 0;
				self.failure_count = 0;
				children.halt(runtime)?;

				return Ok(state);
			}
		}

		Ok(BehaviorState::Running)
	}

	fn provided_ports() -> PortList {
		port_list![
			input_port!(i32, SUCCESS_COUNT),
			input_port!(i32, FAILURE_COUNT)
		]
	}
}
// endregion:   --- Parallel
