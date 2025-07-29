// Copyright © 2025 Stephan Kunz

//! [`BehaviorTree`] implementation.

#[doc(hidden)]
#[cfg(feature = "std")]
extern crate std;

// region:      --- modules
#[cfg(feature = "std")]
use alloc::string::String;
use alloc::{sync::Arc, vec::Vec};
use libloading::Library;
use parking_lot::Mutex;
use tinyscript::SharedRuntime;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    behavior::{BehaviorError, BehaviorResult, BehaviorState},
    blackboard::SharedBlackboard,
    factory::BehaviorRegistry,
    tree::{
        observer::groot2_connector::{GROOT_STATE, Groot2ConnectorData, attach_groot_callback},
        tree_iter::{TreeIter, TreeIterMut},
    },
};

use super::{BehaviorTreeElement, error::Error};
// endregion:   --- modules

// region:		--- helper
/// Helper function to print a (sub)tree recursively
/// # Errors
/// - if recursion is deeper than 127
#[cfg(feature = "std")]
pub fn print_tree(start_node: &BehaviorTreeElement) -> Result<(), Error> {
    print_recursively(0, start_node)
}

/// Recursion function to print a (sub)tree recursively
/// # Errors
/// - Limit is a tree-depth of 127
#[cfg(feature = "std")]
fn print_recursively(level: i8, node: &BehaviorTreeElement) -> Result<(), Error> {
    if level == i8::MAX {
        return Err(Error::Unexpected(
            "recursion limit reached".into(),
            file!().into(),
            line!(),
        ));
    }

    let next_level = level + 1;
    let mut indentation = String::new();
    for _ in 0..level {
        indentation.push_str("  ");
    }

    std::println!("{indentation}{}", node.data().description().name());
    for child in &**node.children() {
        print_recursively(next_level, child)?;
    }
    Ok(())
}
// endregion:	--- helper

// region:      --- BehaviorTreeMessage
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
    uuid: Uuid,
    /// The root element
    root: BehaviorTreeElement,
    /// `runtime` is shared between elements
    runtime: SharedRuntime,
    /// `libraries` stores a reference to the used shared libraries aka plugins.
    /// This is necessary to avoid memory deallocation of libs while tree is in use.
    _libraries: Vec<Arc<Library>>,
    /// The sender, to be cloned on purpose
    tx: mpsc::Sender<BehaviorTreeMessage>,
    /// The receiver
    rx: mpsc::Receiver<BehaviorTreeMessage>,
}

impl BehaviorTree {
    /// create a Tree with reference to its libraries
    #[must_use]
    pub fn new(root: BehaviorTreeElement, registry: &BehaviorRegistry) -> Self {
        // create a clone of the scripting runtime
        let runtime = Arc::new(Mutex::new(registry.runtime().clone()));
        // clone the current state of registered libraries
        let mut libraries = Vec::new();
        for lib in registry.libraries() {
            libraries.push(lib.clone());
        }

        let (tx, rx) = mpsc::channel::<BehaviorTreeMessage>(10);
        Self {
            uuid: Uuid::new_v4(),
            root,
            runtime,
            _libraries: libraries,
            tx,
            rx,
        }
    }

    /// Access the root blackboard of the tree.
    #[must_use]
    pub const fn blackboard(&self) -> &SharedBlackboard {
        self.root.data().blackboard()
    }

    /// Access the root blackboard of the tree.
    #[must_use]
    pub const fn blackboard_mut(&mut self) -> &mut SharedBlackboard {
        self.root.data_mut().blackboard_mut()
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
    pub fn subtree(&self, _index: usize) -> Result<&BehaviorTreeElement, Error> {
        todo!("subtree access")
    }

    /// Get the trees uuid.
    #[must_use]
    pub const fn uuid(&self) -> Uuid {
        self.uuid
    }

    /// Get a message sender.
    /// This sender can be used to modify the tree while running.
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

    /// Handle incomming message    
    #[allow(clippy::redundant_locals)]
    fn handle_message(&mut self, message: BehaviorTreeMessage) {
        let message = message;
        match message {
            BehaviorTreeMessage::RemoveAllGrootHooks => {
                for element in self.iter_mut() {
                    element.remove_pre_state_change_callback(&GROOT_STATE.into());
                }
            }
            BehaviorTreeMessage::AddGrootCallback(data) => {
                attach_groot_callback(self, data);
            }
        }
    }

    /// Ticks the tree exactly once.
    /// # Errors
    #[inline]
    pub async fn tick_exactly_once(&mut self) -> BehaviorResult {
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
            if let Ok(message) = self.rx.try_recv() {
                self.handle_message(message);
            }
            state = self.root.tick(&self.runtime).await?;

            // Not implemented: Check for wake-up conditions and tick again if so
            // Not sure if this is still necessary with real async
            // @TODO!
        }

        // be cooperative & allow pending tasks to catch up
        // crucial for spawned tasks with bounded channels
        tokio::task::yield_now().await;

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
