// Copyright © 2025 Stephan Kunz

//! `ParallelAll` behavior implementation
//!

// region:      --- modules
use alloc::boxed::Box;
use alloc::collections::btree_set::BTreeSet;
use tinyscript::SharedRuntime;

use crate as behaviortree;
use crate::{
	Control, IDLE, MAX_FAILURES,
	behavior::{BehaviorData, Behavior, BehaviorResult, BehaviorState, error::BehaviorError},
	port::PortList,
	tree::tree_element_list::ConstBehaviorTreeElementList,
};
use crate::{input_port, port_list};
// endregion:   --- modules

// region:      --- ParallelAll
/// A `ParallelAll` executes its children
///
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
