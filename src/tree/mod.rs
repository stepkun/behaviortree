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
pub use tree_element_list::BehaviorTreeElementList;

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tree::tree_iter::TreeIter;

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<error::Error>();
		is_normal::<BehaviorTree>();
		is_normal::<BehaviorTreeElement>();
		is_normal::<BehaviorTreeElementList>();
		is_normal::<TreeIter>();
	}
}
