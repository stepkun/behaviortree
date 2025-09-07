// Copyright © 2025 Stephan Kunz
//! [`ChangeStateAfter`] [`Action`] implementation.

// region:      --- modules
use alloc::boxed::Box;
use tinyscript::SharedRuntime;

use crate::{
	self as behaviortree, Action,
	behavior::{Behavior, BehaviorData, BehaviorError, BehaviorResult, BehaviorState},
	tree::ConstBehaviorTreeElementList,
};
//endregion:    --- modules

// region:		--- ChangeStateAfter
/// The `ChangeStateAfter` behavior returns
/// - the stored [`BehaviorState`] `state2` after the amount of ticks given by `count`,
/// - the [`BehaviorState`] `state1` just one tick before reaching `count`,
/// - before that the [`BehaviorState::Running`].
///
/// This [`Behavior`] is used to provide the [`Action`]s that return a certain response after a certain amount of ticks like
/// `AlwaysFailure` and `AlwaysSuccess`.
/// The registration is not possible via the providedd functions or macros, butmust be done manually using its new(...) method like:
/// ```no-test
/// let bhvr_desc = BehaviorDescription::new(
///     "AlwaysSkipped",
///     "AlwaysSkipped",
///     ChangeStateAfter::kind(),
///     false,                         // true, if it is a builtin behavior by Groot2
///     ChangeStateAfter::provided_ports(),
/// );
/// let bhvr_creation_fn =
///     Box::new(move || -> Box<dyn BehaviorExecution> { Box::new(ChangeStateAfter::new(BehaviorState::Running, BehaviorState::Skipped, 0)) });
/// self.registry_mut()
///     add_behavior(bhvr_desc, bhvr_creation_fn)?;
/// ```
#[derive(Action, Debug, Default)]
pub struct ChangeStateAfter {
	/// The [`BehaviorState`] to return initially.
	state1: BehaviorState,
	/// The [`BehaviorState`] to return finally.
	state2: BehaviorState,
	/// The amount of ticks after which the state2 will be returned.
	count: u8,
	/// The remaining ticks until state2 will be returned.
	remaining: u8,
}

#[async_trait::async_trait]
impl Behavior for ChangeStateAfter {
	fn on_halt(&mut self) -> Result<(), BehaviorError> {
		self.remaining = self.count;
		Ok(())
	}

	fn on_start(
		&mut self,
		_behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		self.remaining = self.count;
		Ok(())
	}

	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		Ok(if self.remaining == 0 {
			self.state2
		} else if self.remaining == 1 {
			self.remaining -= 1;
			self.state1
		} else {
			self.remaining -= 1;
			BehaviorState::Running
		})
	}
}

impl ChangeStateAfter {
	/// Constructor with arguments.
	#[must_use]
	pub const fn new(state1: BehaviorState, state2: BehaviorState, count: u8) -> Self {
		Self {
			state1,
			state2,
			count,
			remaining: count,
		}
	}

	/// Initialization function.
	pub const fn initialize(&mut self, state1: BehaviorState, state2: BehaviorState, count: u8) {
		self.state1 = state1;
		self.state2 = state2;
		self.count = count;
		self.remaining = count;
	}
}
// endregion:	--- ChangeStateAfter
