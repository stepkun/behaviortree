// Copyright © 2025 Stephan Kunz
#![allow(clippy::unused_async)]

//! [`BehaviorTree`] implementation.
//!
//! Implemenation is a [`composite pattern`](https://en.wikipedia.org/wiki/Composite_pattern)
//! together with a [`proxy pattern`](https://en.wikipedia.org/wiki/Proxy_pattern)
//!

#[doc(hidden)]
#[cfg(feature = "std")]
extern crate std;

// region:      --- modules
use alloc::{vec, vec::Vec};
use core::marker::PhantomData;

use super::tree_element::BehaviorTreeElement;
// endregion:   --- modules

// region:		--- TreeIter
/// Iterator over the [`BehaviorTree`]
pub struct TreeIter<'a> {
    /// stack to do a depth first search
    stack: Vec<&'a BehaviorTreeElement>,
    /// Lifetime marker
    marker: PhantomData<&'a BehaviorTreeElement>,
}

impl<'a> TreeIter<'a> {
    /// Create a new tree iterator.
    #[must_use]
    pub fn new(root: &'a BehaviorTreeElement) -> Self {
        Self {
            stack: vec![root],
            marker: PhantomData,
        }
    }
}

impl<'a> Iterator for TreeIter<'a> {
    type Item = &'a BehaviorTreeElement;

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_wrap)]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            // Push children in revers order to maintain left-to-right order
            for child in node.children_iter().rev() {
                self.stack.push(child);
            }
            return Some(node);
        }
        None
    }
}
// endregion:	--- TreeIter

// region:		--- TreeIterMut
/// Mutable Iterator over the [`BehaviorTree`]
pub struct TreeIterMut<'a> {
    /// stack to do a depth first search
    stack: Vec<*mut BehaviorTreeElement>,
    /// Lifetime marker
    marker: PhantomData<&'a mut BehaviorTreeElement>,
}

impl<'a> TreeIterMut<'a> {
    /// Create a new mutable tree iterator.
    #[must_use]
    pub fn new(root: &'a mut BehaviorTreeElement) -> Self {
        Self {
            stack: vec![root],
            marker: PhantomData,
        }
    }
}

#[allow(unsafe_code)]
impl<'a> Iterator for TreeIterMut<'a> {
    type Item = &'a mut BehaviorTreeElement;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node_ptr) = self.stack.pop() {
            // we know this pointer is valid since the iterator owns the traversal
            let node = unsafe { &mut *node_ptr };
            // Push children in revers order to maintain left-to-right order
            for child in node.children_iter_mut().rev() {
                self.stack.push(child);
            }
            return Some(&mut *node);
        }
        None
    }
}
// endregion:	--- TreeIterMut
