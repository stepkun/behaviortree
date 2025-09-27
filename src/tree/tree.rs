// Copyright © 2025 Stephan Kunz

//! [`BehaviorTree`] implementation.

#[doc(hidden)]
#[cfg(feature = "std")]
extern crate std;

// region:      --- modules
#[cfg(feature = "std")]
use crate::tree::observer::groot2_connector::{GROOT_STATE, Groot2ConnectorData, attach_groot_callback};
use crate::{
	behavior::{BehaviorError, BehaviorResult, BehaviorState},
	factory::BehaviorRegistry,
	tree::{
		tree_element::TreeElementKind,
		tree_iter::{TreeIter, TreeIterMut},
	},
};
#[cfg(feature = "std")]
use alloc::vec::Vec;
use alloc::{
	string::{String, ToString},
	sync::Arc,
};
use databoard::Databoard;
#[cfg(feature = "std")]
use libloading::Library;
use spin::Mutex;
use tinyscript::SharedRuntime;
#[cfg(feature = "std")]
use tokio::sync::mpsc;
#[cfg(feature = "std")]
use uuid::Uuid;

use super::{error::Error, tree_element::BehaviorTreeElement};
// endregion:   --- modules

// region:		--- helper
/// Recursion function to print a (sub)tree recursively, limit is a tree-depth of 127
/// # Errors
/// - if limit of 127 for tree depth is exceeded
fn print_recursively(level: i8, node: &BehaviorTreeElement) -> Result<(), Error> {
	if level == i8::MAX {
		return Err(Error::RecursionLimit(node.data().description().name().clone()));
	}

	let next_level = level + 1;
	let mut indentation = String::new();
	for _ in 0..level {
		indentation.push_str("  ");
	}

	#[cfg(feature = "std")]
	std::println!("{indentation}{}", node.data().description().name());
	for child in &**node.children() {
		print_recursively(next_level, child)?;
	}
	Ok(())
}
// endregion:	--- helper

// region:      --- BehaviorTreeMessage
#[cfg(feature = "std")]
pub enum BehaviorTreeMessage {
	AddGrootCallback(Arc<Mutex<Groot2ConnectorData>>),
	RemoveAllGrootHooks,
}
// endregion:   --- BehaviorTreeMessage

// region:		--- BehaviorTree
/// A Tree of [`BehaviorTreeElement`]s.
/// A certain [`BehaviorTree`] can contain up to 65536 [`BehaviorTreeElement`]s.
pub struct BehaviorTree {
	/// The trees unique id
	#[cfg(feature = "std")]
	uuid: Uuid,
	/// The root element
	root: BehaviorTreeElement,
	/// `runtime` is shared between elements
	runtime: SharedRuntime,
	/// `libraries` stores a reference to the used shared libraries aka plugins.
	/// This is necessary to avoid memory deallocation of libs while tree is in use.
	#[cfg(feature = "std")]
	_libraries: Vec<Arc<Library>>,
	/// The sender, to be cloned on purpose
	#[cfg(feature = "std")]
	tx: mpsc::Sender<BehaviorTreeMessage>,
	/// The receiver
	#[cfg(feature = "std")]
	rx: mpsc::Receiver<BehaviorTreeMessage>,
}

impl BehaviorTree {
	/// create a Tree with reference to its libraries
	#[must_use]
	pub fn new(root: BehaviorTreeElement, registry: &BehaviorRegistry) -> Self {
		// create a [`SharedRuntime`](https://docs.rs/tinyscript/latest/tinyscript/runtime/type.SharedRuntime.html)
		// based on the current state of registriesscripting runtime
		let runtime = Arc::new(Mutex::new(registry.runtime().clone()));
		// clone the current state of registered libraries so that they are not deallocated while tree is running
		#[cfg(feature = "std")]
		let mut libraries = Vec::with_capacity(registry.libraries().capacity() + 1);
		#[cfg(feature = "std")]
		for lib in registry.libraries() {
			libraries.push(lib.clone());
		}

		#[cfg(feature = "std")]
		let (tx, rx) = mpsc::channel::<BehaviorTreeMessage>(10);
		Self {
			#[cfg(feature = "std")]
			uuid: Uuid::new_v4(),
			root,
			runtime,
			#[cfg(feature = "std")]
			_libraries: libraries,
			#[cfg(feature = "std")]
			tx,
			#[cfg(feature = "std")]
			rx,
		}
	}

	/// Access the root blackboard of the tree.
	#[must_use]
	pub const fn blackboard(&self) -> &Databoard {
		self.root.data().blackboard()
	}

	/// Pretty print the tree.
	/// # Errors
	/// - if tree depth exceeds 127 (sub)tree levels.
	#[inline]
	pub fn print(&self) -> Result<(), Error> {
		print_recursively(0, &self.root)
	}

	/// Get a (sub)tree where index 0 is root tree.
	/// # Errors
	/// - if index is out of bounds.
	pub fn subtree(&self, index: usize) -> Result<&BehaviorTreeElement, Error> {
		let mut i = 0_usize;
		for element in self.iter() {
			if matches!(element.kind(), TreeElementKind::SubTree) {
				if i == index {
					return Ok(element);
				}
				i += 1;
			}
		}
		Err(Error::SubtreeNotFound(index.to_string().into()))
	}

	/// Get the trees uuid.
	#[cfg(feature = "std")]
	#[must_use]
	pub const fn uuid(&self) -> Uuid {
		self.uuid
	}

	/// Get a message sender.
	/// This sender can be used to modify the tree while running.
	#[cfg(feature = "std")]
	#[must_use]
	pub fn sender(&self) -> mpsc::Sender<BehaviorTreeMessage> {
		self.tx.clone()
	}

	/// Get the trees total number of children.
	#[must_use]
	pub fn size(&self) -> u16 {
		let mut count = 0;
		let iter = self.iter();
		for _ in iter {
			count += 1;
		}
		count
	}

	/// Handle incoming message    
	#[cfg(feature = "std")]
	fn handle_message(&mut self, message: BehaviorTreeMessage) {
		match message {
			BehaviorTreeMessage::RemoveAllGrootHooks => {
				// std::dbg!("removing all Groot hooks");
				for element in self.iter_mut() {
					element.remove_pre_state_change_callback(&GROOT_STATE.into());
				}
			}
			BehaviorTreeMessage::AddGrootCallback(data) => {
				// std::dbg!("adding Groot callback");
				attach_groot_callback(self, data);
			}
		}
	}

	/// Ticks the tree exactly once.
	/// # Errors
	#[inline]
	pub async fn tick_exactly_once(&mut self) -> BehaviorResult {
		#[cfg(feature = "std")]
		if let Ok(message) = self.rx.try_recv() {
			self.handle_message(message);
		}
		self.root.tick(&self.runtime).await
	}

	/// Ticks the tree once.
	/// @TODO: The wakeup mechanism is not yet implemented
	/// # Errors
	#[inline]
	pub async fn tick_once(&mut self) -> BehaviorResult {
		#[cfg(feature = "std")]
		if let Ok(message) = self.rx.try_recv() {
			self.handle_message(message);
		}
		self.root.tick(&self.runtime).await
	}

	/// Ticks the tree until it finishes either with [`BehaviorState::Success`] or [`BehaviorState::Failure`].
	/// # Errors
	pub async fn tick_while_running(&mut self) -> BehaviorResult {
		let mut state = BehaviorState::Running;
		while state == BehaviorState::Running || state == BehaviorState::Idle {
			#[cfg(feature = "std")]
			while let Ok(message) = self.rx.try_recv() {
				self.handle_message(message);
			}
			state = self.root.tick(&self.runtime).await?;

			// Not implemented: Check for wake-up conditions and tick again if so
			// Not sure if this is still necessary with real async
			// @TODO!
		}

		// be cooperative & allow pending tasks to catch up
		// crucial for spawned tasks with bounded channels
		#[cfg(feature = "std")]
		{
			tokio::task::yield_now().await;
		}

		// handle eventually pending messages
		#[cfg(feature = "std")]
		while let Ok(message) = self.rx.try_recv() {
			self.handle_message(message);
		}
		Ok(state)
	}

	/// Get an iterator over the tree.
	#[inline]
	pub fn iter(&self) -> impl Iterator<Item = &BehaviorTreeElement> {
		TreeIter::new(&self.root)
	}

	/// Get a mutable iterator over the tree.
	#[inline]
	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut BehaviorTreeElement> {
		TreeIterMut::new(&mut self.root)
	}

	/// Reset tree to initial state.
	/// # Errors
	/// - if reset of children failed
	pub fn reset(&mut self) -> Result<(), BehaviorError> {
		self.root.halt(&self.runtime)?;
		self.runtime.lock().clear();
		Ok(())
	}
}
// endregion:	--- BehaviorTree
