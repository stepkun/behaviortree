// Copyright © 2025 Stephan Kunz

//! A [`BehaviorTree`](crate::tree::tree::BehaviorTree) element.

// region:      --- modules
use crate::{
	ConstString, FAILURE_IF, ON_FAILURE, ON_SUCCESS, POST, SKIP_IF, SUCCESS_IF, WHILE,
	behavior::BehaviorDataCollection,
	behavior::{
		BehaviorPtr, BehaviorResult, BehaviorState,
		behavior_data::BehaviorData,
		error::Error as BehaviorError,
		pre_post_conditions::{Conditions, PostConditions, PreConditions},
	},
	tree::{
		tree_element_list::BehaviorTreeElementList,
		tree_iter::{TreeIter, TreeIterMut},
	},
};
use alloc::{boxed::Box, string::ToString};
use databoard::{Databoard, Remappings};
use tinyscript::{Error, SharedRuntime};
// endregion:   --- modules

// region:		--- TreeElementKind
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
/// The different kinds of a [`BehaviorTreeElement`]
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
	children: BehaviorTreeElementList,
	/// Tuple of pre- and post-conditions, checked before and after a tick.
	conditions: Conditions,
}

impl BehaviorTreeElement {
	/// Construct a [`BehaviorTreeElement`].
	#[must_use]
	pub fn new(
		kind: TreeElementKind,
		behavior: BehaviorPtr,
		mut data: BehaviorData,
		children: BehaviorTreeElementList,
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
			conditions,
		}
	}

	/// Create a tree leaf.
	#[must_use]
	pub(crate) fn create_leaf(data: Box<BehaviorDataCollection>) -> Self {
		let bhvr_data = BehaviorData::new(&data);
		Self::new(
			TreeElementKind::Leaf,
			data.bhvr,
			bhvr_data,
			BehaviorTreeElementList::default(),
			data.conditions,
		)
	}

	/// Create a tree node.
	#[must_use]
	pub(crate) fn create_node(data: Box<BehaviorDataCollection>, children: BehaviorTreeElementList) -> Self {
		let bhvr_data = BehaviorData::new(&data);
		Self::new(TreeElementKind::Node, data.bhvr, bhvr_data, children, data.conditions)
	}

	/// Create a subtree.
	#[must_use]
	pub(crate) fn create_subtree(data: Box<BehaviorDataCollection>, children: BehaviorTreeElementList) -> Self {
		let bhvr_data = BehaviorData::new(&data);
		Self::new(TreeElementKind::SubTree, data.bhvr, bhvr_data, children, data.conditions)
	}

	/// Get the uid.
	#[must_use]
	pub const fn uid(&self) -> u16 {
		self.data.uid()
	}

	/// Get the id.
	#[must_use]
	pub const fn id(&self) -> &ConstString {
		self.data.description().id()
	}

	/// Returns the name of the behavior.
	#[must_use]
	pub const fn name(&self) -> &ConstString {
		self.data.description().name()
	}

	/// Returns a reference to the [`BehaviorData`].
	#[must_use]
	pub const fn data(&self) -> &BehaviorData {
		&self.data
	}

	/// Returns a mutable reference to the [`BehaviorData`].
	#[must_use]
	pub const fn data_mut(&mut self) -> &mut BehaviorData {
		&mut self.data
	}

	/// Returns a reference to the behavior.
	#[must_use]
	pub const fn behavior(&self) -> &BehaviorPtr {
		&self.behavior
	}

	/// Returns a mutable reference to the behavior.
	#[must_use]
	pub const fn behavior_mut(&mut self) -> &mut BehaviorPtr {
		&mut self.behavior
	}

	/// Returns a reference to the blackboard.
	#[must_use]
	pub const fn blackboard(&self) -> &Databoard {
		self.data().blackboard()
	}

	/// Returns the children.
	#[must_use]
	pub const fn children(&self) -> &BehaviorTreeElementList {
		&self.children
	}

	/// Returns the children mutable.
	#[must_use]
	pub const fn children_mut(&mut self) -> &mut BehaviorTreeElementList {
		&mut self.children
	}

	/// Returns the pre conditions.
	#[must_use]
	pub const fn pre_conditions(&self) -> &PreConditions {
		&self.conditions.pre
	}

	/// Returns the post conditions.
	#[must_use]
	pub const fn post_conditions(&self) -> &PostConditions {
		&self.conditions.post
	}

	/// Returns the remappings.
	#[must_use]
	pub const fn remappings(&self) -> &Remappings {
		self.data.remappings()
	}

	/// Halts the element and all its children considering postconditions.
	/// # Errors
	pub fn halt(&mut self, runtime: &SharedRuntime) -> Result<(), BehaviorError> {
		if self.data.state() != BehaviorState::Idle {
			let state = self
				.behavior
				.halt(&mut self.data, &mut self.children, runtime)?;
			self.data.set_state(state);
			if let Some(script) = self.conditions.post.get("_onHalted") {
				let _ = runtime.lock().run(script, &mut self.data)?;
			}
		}
		Ok(())
	}

	/// Ticks the element considering pre- and postconditions.
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

		self.check_post_conditions(state, runtime)?;

		// Preserve the last state if skipped, but communicate `Skipped` to parent
		if state != BehaviorState::Skipped {
			self.data.set_state(state);
			// } else {
			//     self.data.set_state(old_state);
		}

		Ok(state)
	}

	/// Halts child at `index`.
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

	/// Reset state of element.
	pub fn reset_state(&mut self) {
		// let prev_state = self.data.state();
		self.data.set_state(BehaviorState::Idle);
		// if prev_state != BehaviorState::Idle {
		// 	// @TODO: tree_node.cpp TreNode::resetStatus()
		// 	todo!()
		// }
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
		if self.conditions.pre.is_some() {
			// Preconditions only applied when the node state is `Idle` or `Skipped`
			if self.data.state() == BehaviorState::Idle || self.data.state() == BehaviorState::Skipped {
				if let Some(script) = self.conditions.pre.get(FAILURE_IF) {
					let res = runtime.lock().run(script, &mut self.data)?;
					if bool::try_from(res)? {
						return Ok(Some(BehaviorState::Failure));
					}
				}
				if let Some(script) = self.conditions.pre.get(SUCCESS_IF) {
					let res = runtime.lock().run(script, &mut self.data)?;
					if bool::try_from(res)? {
						return Ok(Some(BehaviorState::Success));
					}
				}
				if let Some(script) = self.conditions.pre.get(SKIP_IF) {
					let res = runtime.lock().run(script, &mut self.data)?;
					if bool::try_from(res)? {
						return Ok(Some(BehaviorState::Skipped));
					}
				}
				if let Some(script) = self.conditions.pre.get(WHILE) {
					let res = runtime.lock().run(script, &mut self.data)?;
					if bool::try_from(res)? {
						return Ok(Some(BehaviorState::Skipped));
					}
				}
			} else
			// Preconditions only applied when the node state is `Running`
			if self.data.state() == BehaviorState::Running
				&& let Some(script) = self.conditions.pre.get(WHILE)
			{
				let res = runtime.lock().run(script, &mut self.data)?;
				// if not true halt element and return `Skipped`
				if bool::try_from(res)? {
					let _res = self.halt(runtime);
					return Ok(Some(BehaviorState::Skipped));
				}
			}
		}
		Ok(None)
	}

	fn check_post_conditions(&mut self, state: BehaviorState, runtime: &SharedRuntime) -> Result<(), Error> {
		if self.conditions.post.is_some() {
			match state {
				BehaviorState::Failure => {
					if let Some(script) = self.conditions.post.get(ON_FAILURE) {
						let _ = runtime.lock().run(script, &mut self.data)?;
					}
				}
				BehaviorState::Success => {
					if let Some(script) = self.conditions.post.get(ON_SUCCESS) {
						let _ = runtime.lock().run(script, &mut self.data)?;
					}
				}
				// rest is ignored
				_ => {}
			}
			if let Some(script) = self.conditions.post.get(POST) {
				let _ = runtime.lock().run(script, &mut self.data)?;
			}
		}
		Ok(())
	}

	/// Returns the full 'path' of the element.
	#[must_use]
	pub const fn full_path(&self) -> &ConstString {
		self.data.description().path()
	}

	/// Returns the Groot2 style 'path' of the element.
	#[must_use]
	pub const fn groot2_path(&self) -> &ConstString {
		self.data.description().groot2_path()
	}

	/// Returns the current state of the element.
	#[must_use]
	pub const fn state(&self) -> BehaviorState {
		self.data.state()
	}
}
// endregion:	--- BehaviorTreeElement
