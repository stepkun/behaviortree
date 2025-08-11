// Copyright © 2025 Stephan Kunz

//! [`BehaviorTreeElementList`] implementation.
//!

// region:      --- modules
use alloc::{boxed::Box, format, vec::Vec};
use core::ops::{Deref, DerefMut};
use tinyscript::SharedRuntime;

use crate::behavior::error::BehaviorError;

use super::tree_element::BehaviorTreeElement;
// endregion:   --- modules

// region:		--- ConstBehaviorTreeElementList
/// An immutable list of tree components.
#[derive(Default)]
pub struct ConstBehaviorTreeElementList(Box<[BehaviorTreeElement]>);

impl Deref for ConstBehaviorTreeElementList {
    type Target = [BehaviorTreeElement];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ConstBehaviorTreeElementList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<BehaviorTreeElementList> for ConstBehaviorTreeElementList {
    fn from(list: BehaviorTreeElementList) -> Self {
        Self(list.0.into_boxed_slice())
    }
}

impl ConstBehaviorTreeElementList {
    /// Reset all children
    /// # Errors
    /// - if a child errors on `halt()`
    pub fn halt(&mut self, runtime: &SharedRuntime) -> Result<(), BehaviorError> {
        for child in &mut self.0 {
            child.halt(runtime)?;
        }
        Ok(())
    }

    /// Halt child at and beyond index.
    /// # Errors
    /// - if halt of a child fails
    pub fn halt_from(
        &mut self,
        index: usize,
        runtime: &SharedRuntime,
    ) -> Result<(), BehaviorError> {
        for i in index..self.0.len() {
            self.0[i].halt(runtime)?;
        }
        Ok(())
    }

    /// Halt child at index.
    /// # Errors
    /// - if index is out of bounds
    /// - if halt of the child fails
    pub fn halt_at(&mut self, index: usize, runtime: &SharedRuntime) -> Result<(), BehaviorError> {
        // An empty children list is ok for Action and Condition
        if self.0.is_empty() {
            return Ok(());
        } else if index >= self.0.len() {
            let txt = format!("behavior tries to halt a non-existent child at index [{index}]");
            return Err(BehaviorError::Composition(txt.into()));
        }
        self.0[index].halt(runtime)
    }
}
// endregion:   --- ConstBehaviorTreeElementList

// region:		--- BehaviorTreeElementList
/// A mutable list of tree components.
#[derive(Default)]
pub struct BehaviorTreeElementList(Vec<BehaviorTreeElement>);

impl Deref for BehaviorTreeElementList {
    type Target = Vec<BehaviorTreeElement>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BehaviorTreeElementList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ConstBehaviorTreeElementList> for BehaviorTreeElementList {
    fn from(list: ConstBehaviorTreeElementList) -> Self {
        Self(list.0.into_vec())
    }
}

impl BehaviorTreeElementList {
    /// Reset all children
    /// # Errors
    /// - if a child errors on `halt()`
    pub fn halt(&mut self, runtime: &SharedRuntime) -> Result<(), BehaviorError> {
        for child in &mut self.0 {
            child.halt(runtime)?;
        }
        Ok(())
    }

    /// Halt child at and beyond index.
    /// # Errors
    /// - if halt of a child fails
    pub fn halt_from(
        &mut self,
        index: usize,
        runtime: &SharedRuntime,
    ) -> Result<(), BehaviorError> {
        for i in index..self.0.len() {
            self.0[i].halt(runtime)?;
        }
        Ok(())
    }

    /// Halt child at index.
    /// # Errors
    /// - if index is out of bounds
    /// - if halt of the child fails
    pub fn halt_at(&mut self, index: usize, runtime: &SharedRuntime) -> Result<(), BehaviorError> {
        // An empty children list is ok for Action and Condition
        if self.0.is_empty() {
            return Ok(());
        } else if index >= self.0.len() {
            let txt = format!("behavior tries to halt a non-existent child at index [{index}]");
            return Err(BehaviorError::Composition(txt.into()));
        }
        self.0[index].halt(runtime)
    }
}
// endregion:	--- BehaviorTreeElementList
