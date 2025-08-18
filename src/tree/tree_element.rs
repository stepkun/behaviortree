// Copyright © 2025 Stephan Kunz

//! A [`BehaviorTree`](crate::tree::tree::BehaviorTree) element.

// region:      --- modules
use alloc::string::ToString;
use tinyscript::{Error, SharedRuntime};

use crate::blackboard::SharedBlackboard;
use crate::tree::tree_iter::TreeIterMut;
use crate::{ConstString, FAILURE_IF, ON_FAILURE, ON_SUCCESS, POST, SKIP_IF, SUCCESS_IF, WHILE};
use crate::{
	behavior::{
		BehaviorData, BehaviorPtr, BehaviorResult, BehaviorState,
		error::BehaviorError,
		pre_post_conditions::{Conditions, PostConditions, PreConditions},
	},
	tree::tree_iter::TreeIter,
};

use super::tree_element_list::ConstBehaviorTreeElementList;
// endregion:   --- modules

// region:		--- TreeElementKind
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
/// @TODO:
pub enum TreeElementKind {
	/// A behavior tree leaf.
	Leaf,
	/// A behavior tree node.
	Node,
	/// A behavior subtree.
	SubTree,
}
//endregion:	--- TreeElementKind

// region:		--- BehaviorTreeElement
/// A tree elements.
pub struct BehaviorTreeElement {
	/// Kind of the element.
	kind: TreeElementKind,
	/// The behavior of that element.
	behavior: BehaviorPtr,
	/// Data of the Behavior.
	data: BehaviorData,
	/// Children of the element.
	children: ConstBehaviorTreeElementList,
	/// Pre conditions, checked before a tick.
	pre_conditions: PreConditions,
	/// Post conditions, checked after a tick.
	post_conditions: PostConditions,
}

impl BehaviorTreeElement {
	/// Construct a [`BehaviorTreeElement`].
	/// Non public to enforce using the dedicated creation functions.
	#[allow(clippy::too_many_arguments)]
	fn new(
		kind: TreeElementKind,
		behavior: BehaviorPtr,
		mut data: BehaviorData,
		children: ConstBehaviorTreeElementList,
		conditions: Conditions,
	) -> Self {
		let groot2_path = match kind {
			TreeElementKind::Leaf | TreeElementKind::Node => data.description().path().clone(),
			TreeElementKind::SubTree => {
				if data.description().path().is_empty() {
					data.description().path().clone()
				} else {
					let uid = data.uid().to_string();
					(data.description().name().to_string() + "::" + &uid).into()
				}
			}
		};
		data.description_mut()
			.set_groot2_path(groot2_path);
		Self {
			kind,
			behavior,
			data,
			children,
			pre_conditions: conditions.pre,
			post_conditions: conditions.post,
		}
	}

	/// Create a tree leaf.
	#[must_use]
	pub(crate) fn create_leaf(data: BehaviorData, behavior: BehaviorPtr, conditions: Conditions) -> Self {
		Self::new(
			TreeElementKind::Leaf,
			behavior,
			data,
			ConstBehaviorTreeElementList::default(),
			conditions,
		)
	}

	/// Create a tree node.
	#[must_use]
	pub(crate) fn create_node(
		data: BehaviorData,
		children: ConstBehaviorTreeElementList,
		behavior: BehaviorPtr,
		conditions: Conditions,
	) -> Self {
		Self::new(TreeElementKind::Node, behavior, data, children, conditions)
	}

	/// Create a subtree.
	#[must_use]
	pub(crate) fn create_subtree(
		data: BehaviorData,
		children: ConstBehaviorTreeElementList,
		behavior: BehaviorPtr,
		conditions: Conditions,
	) -> Self {
		Self::new(TreeElementKind::SubTree, behavior, data, children, conditions)
	}

	/// Get the uid.
	#[must_use]
	pub const fn uid(&self) -> u16 {
		self.data.uid()
	}

	/// Get a reference to the [`BehaviorData`].
	#[must_use]
	pub const fn data(&self) -> &BehaviorData {
		&self.data
	}

	/// Get a mutable reference to the [`BehaviorData`].
	#[must_use]
	pub const fn data_mut(&mut self) -> &mut BehaviorData {
		&mut self.data
	}

	/// Get a reference to the behavior.
	#[must_use]
	pub const fn behavior(&self) -> &BehaviorPtr {
		&self.behavior
	}

	/// Get a mutable reference to the behavior.
	#[must_use]
	pub const fn behavior_mut(&mut self) -> &mut BehaviorPtr {
		&mut self.behavior
	}

	/// Get a reference to the blackboard.
	#[must_use]
	pub const fn blackboard(&self) -> &SharedBlackboard {
		self.data().blackboard()
	}

	/// Get a mutable reference to the blackboard.
	#[must_use]
	pub const fn blackboard_mut(&mut self) -> &mut SharedBlackboard {
		self.data_mut().blackboard_mut()
	}

	/// Get the children.
	#[must_use]
	pub const fn children(&self) -> &ConstBehaviorTreeElementList {
		&self.children
	}

	/// Get the children mutable.
	#[must_use]
	pub const fn children_mut(&mut self) -> &mut ConstBehaviorTreeElementList {
		&mut self.children
	}

	/// Get the pre conditions.
	#[must_use]
	pub const fn pre_conditions(&self) -> &PreConditions {
		&self.pre_conditions
	}

	/// Get the post conditions.
	#[must_use]
	pub const fn post_conditions(&self) -> &PostConditions {
		&self.post_conditions
	}

	/// Halt the element and all its children considering postconditions.
	/// # Errors
	pub fn halt(&mut self, runtime: &SharedRuntime) -> Result<(), BehaviorError> {
		if self.data.state() != BehaviorState::Idle {
			let state = self
				.behavior
				.halt(&mut self.data, &mut self.children, runtime)?;
			self.data.set_state(state);
			if let Some(script) = self.post_conditions.get("_onHalted") {
				let _ = runtime
					.lock()
					.run(script, self.data.blackboard_mut())?;
			}
		}
		Ok(())
	}

	/// Tick the element considering pre- and postconditions.
	/// # Errors
	pub async fn tick(&mut self, runtime: &SharedRuntime) -> BehaviorResult {
		// A pre-condition may return the next state which will override the current tick().
		let old_state = self.data.state();
		let state = if let Some(result) = self.check_pre_conditions(runtime)? {
			result
		} else if old_state == BehaviorState::Idle {
			self.behavior
				.start(&mut self.data, &mut self.children, runtime)
				.await?
		} else {
			self.behavior
				.tick(&mut self.data, &mut self.children, runtime)
				.await?
		};

		self.check_post_conditions(state, runtime);

		// Preserve the last state if skipped, but communicate `Skipped` to parent
		if state != BehaviorState::Skipped {
			self.data.set_state(state);
			// } else {
			//     self.data.set_state(old_state);
		}

		Ok(state)
	}

	/// Halt child at `index`.
	/// # Errors
	/// - if index is out of childrens bounds.
	#[inline]
	pub fn halt_child_at(&mut self, index: usize, runtime: &SharedRuntime) -> Result<(), BehaviorError> {
		self.children.halt_at(index, runtime)
	}

	/// Halt all children at and beyond `index`.
	/// # Errors
	/// - if index is out of childrens bounds.
	#[inline]
	pub fn halt_children_from(&mut self, index: usize, runtime: &SharedRuntime) -> Result<(), BehaviorError> {
		self.children.halt_from(index, runtime)
	}

	/// Halt all children at and beyond `index`.
	/// # Errors
	/// - if index is out of childrens bounds.
	#[inline]
	pub fn halt_children(&mut self, runtime: &SharedRuntime) -> Result<(), BehaviorError> {
		self.children.halt(runtime)
	}

	/// Add a pre state change callback with the given name.
	/// The name is not unique, which is important when removing callback.
	#[inline]
	pub fn add_pre_state_change_callback<T>(&mut self, name: ConstString, callback: T)
	where
		T: Fn(&BehaviorData, &mut BehaviorState) + Send + Sync + 'static,
	{
		self.data
			.add_pre_state_change_callback(name, callback);
	}

	/// Remove any pre state change callback with the given name.
	#[inline]
	pub fn remove_pre_state_change_callback(&mut self, name: &ConstString) {
		self.data.remove_pre_state_change_callback(name);
	}

	/// Return an iterator over the children.
	#[must_use]
	#[inline]
	pub fn children_iter(&self) -> impl DoubleEndedIterator<Item = &Self> {
		self.children().iter()
	}

	/// Return a mutable iterator over the children.
	#[must_use]
	#[inline]
	pub fn children_iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Self> {
		self.children_mut().iter_mut()
	}

	/// Get an iterator over the tree element.
	#[inline]
	pub fn iter(&self) -> impl Iterator<Item = &Self> {
		TreeIter::new(self)
	}

	/// Get a mutable iterator over the tree element.
	#[inline]
	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Self> {
		TreeIterMut::new(self)
	}

	pub(crate) const fn kind(&self) -> TreeElementKind {
		self.kind
	}

	fn check_pre_conditions(&mut self, runtime: &SharedRuntime) -> Result<Option<BehaviorState>, Error> {
		if self.pre_conditions.is_some() {
			// Preconditions only applied when the node state is `Idle` or `Skipped`
			if self.data.state() == BehaviorState::Idle || self.data.state() == BehaviorState::Skipped {
				if let Some(script) = self.pre_conditions.get(FAILURE_IF) {
					let res = runtime
						.lock()
						.run(script, self.data.blackboard_mut())?;
					if res.is_bool() && res.as_bool()? {
						return Ok(Some(BehaviorState::Failure));
					}
				}
				if let Some(script) = self.pre_conditions.get(SUCCESS_IF) {
					let res = runtime
						.lock()
						.run(script, self.data.blackboard_mut())?;
					if res.is_bool() && res.as_bool()? {
						return Ok(Some(BehaviorState::Success));
					}
				}
				if let Some(script) = self.pre_conditions.get(SKIP_IF) {
					let res = runtime
						.lock()
						.run(script, self.data.blackboard_mut())?;
					if res.is_bool() && res.as_bool()? {
						return Ok(Some(BehaviorState::Skipped));
					}
				}
				if let Some(script) = self.pre_conditions.get(WHILE) {
					let res = runtime
						.lock()
						.run(script, self.data.blackboard_mut())?;
					if res.is_bool() && res.as_bool()? {
						return Ok(Some(BehaviorState::Skipped));
					}
				}
			} else
			// Preconditions only applied when the node state is `Running`
			if self.data.state() == BehaviorState::Running {
				if let Some(script) = self.pre_conditions.get(WHILE) {
					let res = runtime
						.lock()
						.run(script, self.data.blackboard_mut())?;
					// if not true halt element and return `Skipped`
					if res.is_bool() && !res.as_bool()? {
						let _res = self.halt(runtime);
						return Ok(Some(BehaviorState::Skipped));
					}
				}
			}
		}
		Ok(None)
	}

	fn check_post_conditions(&mut self, state: BehaviorState, runtime: &SharedRuntime) {
		if self.post_conditions.is_some() {
			match state {
				BehaviorState::Failure => {
					if let Some(script) = self.post_conditions.get(ON_FAILURE) {
						let _: Result<tinyscript::execution::ScriptingValue, tinyscript::Error> = runtime
							.lock()
							.run(script, self.data.blackboard_mut());
					}
				}
				BehaviorState::Success => {
					if let Some(script) = self.post_conditions.get(ON_SUCCESS) {
						let _ = runtime
							.lock()
							.run(script, self.data.blackboard_mut());
					}
				}
				// rest is ignored
				_ => {}
			}
			if let Some(script) = self.post_conditions.get(POST) {
				let _ = runtime
					.lock()
					.run(script, self.data.blackboard_mut());
			}
		}
	}
}
// endregion:	--- BehaviorTreeElement
