// Copyright © 2025 Stephan Kunz

//! [`behaviortree`](crate) tree module.

pub mod error;
pub mod observer;
#[allow(clippy::module_inception)]
mod tree;
mod tree_element;
mod tree_element_list;
mod tree_iter;

// flatten
pub use tree::BehaviorTree;
pub use tree_element::{BehaviorTreeElement, TreeElementKind};
pub use tree_element_list::{BehaviorTreeElementList, ConstBehaviorTreeElementList};
