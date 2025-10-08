// Copyright © 2025 Stephan Kunz
//! [`ChangeStateAfter`] is a helper  [`Action`] for writing tests.

#![allow(unused)]

#[doc(hidden)]
extern crate alloc;

use alloc::boxed::Box;
use behaviortree::prelude::*;

/// The `ChangeStateAfter` behavior returns
/// - the stored [`BehaviorState`] `final_state` after the amount of ticks given by `max_count`,
/// - the [`BehaviorState`] `state1` just one tick before reaching `max_count`,
/// - before that the [`BehaviorState::Running`].
///
/// This [`Behavior`] is used to provide the [`Action`]s that return a certain response after a
/// certain amount of ticks like `AlwaysFailure` and `AlwaysSuccess`.
/// The behavior is also used to create test behaviors.
///
/// The registration is possible via the provided macro,
/// ```no-test
/// register_behavior!(factory, ChangeStateAfter, "AlwaysSkipped",
///                        BehaviorState::Running, BehaviorState::Skipped, 0)?;
/// ```
#[derive(Action, Debug)]
pub struct ChangeStateAfter {
	/// The [`BehaviorState`] to one tick before `max_count`.
	state1: BehaviorState,
	/// The [`BehaviorState`] to return finally when reaching `max_count`.
	final_state: BehaviorState,
	/// The amount of ticks after which the state2 will be returned.
	max_count: usize,
	/// The current tick count.
	tick_count: usize,
}

impl Default for ChangeStateAfter {
	fn default() -> Self {
		Self {
			state1: BehaviorState::Running,
			final_state: BehaviorState::Failure,
			max_count: 0,
			tick_count: 0,
		}
	}
}

#[async_trait::async_trait]
impl Behavior for ChangeStateAfter {
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		Ok(())
	}

	fn on_start(
		&mut self,
		_behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		self.tick_count = 0;
		Ok(())
	}

	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		Ok(if self.tick_count == self.max_count {
			self.final_state
		} else if self.max_count - self.tick_count == 1 {
			self.tick_count += 1;
			self.state1
		} else {
			self.tick_count += 1;
			BehaviorState::Running
		})
	}
}

impl ChangeStateAfter {
	/// Returns a [`ChangeStateAfter`] behavior with the given parameters.
	#[must_use]
	pub const fn new(state1: BehaviorState, final_state: BehaviorState, count: usize) -> Self {
		Self {
			state1,
			final_state,
			max_count: count,
			tick_count: 0,
		}
	}

	/// Initialization function.
	pub const fn initialize(&mut self, state1: BehaviorState, final_state: BehaviorState, count: usize) {
		self.state1 = state1;
		self.final_state = final_state;
		self.max_count = count;
		self.tick_count = 0;
	}

	/// Returns the current number of tick's this behavior received.
	#[must_use]
	#[inline]
	pub const fn tick_count(&self) -> usize {
		self.tick_count
	}

	/// Modifies the value for `state1`.
	#[inline]
	pub const fn set_state1(&mut self, state: BehaviorState) {
		self.state1 = state;
	}

	/// Modifies the value for `final_state`.
	#[inline]
	pub const fn set_final_state(&mut self, state: BehaviorState) {
		self.final_state = state;
	}
}
