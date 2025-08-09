// Copyright © 2025 Stephan Kunz

//! Implementation for shared usage of [`BlackboardNode`]s.

#[doc(hidden)]
#[cfg(feature = "std")]
extern crate std;

// region:      --- modules
use crate::ConstString;
use alloc::{
    string::{String, ToString},
    sync::Arc,
};
use core::{
    any::Any,
    fmt::Debug,
    ops::{Deref, DerefMut},
    str::FromStr,
};
use spin::RwLock;
use tinyscript::{Environment, Error as ScriptingError, execution::ScriptingValue};

use super::{BlackboardInterface, blackboard::Blackboard, blackboard_data::Entry, error::Error};

use crate::port::ConstPortRemappings;
// endregion:   --- modules

fn strip_key(key: &str) -> &str {
    if key.starts_with('{') && key.ends_with('}') {
        key.strip_prefix('{')
            .unwrap_or_else(|| todo!())
            .strip_suffix('}')
            .unwrap_or_else(|| todo!())
    } else {
        key
    }
}

// region:      --- SharedBlackboard
/// Thread safe reference to a [`Blackboard`].
#[derive(Clone, Debug, Default)]
pub struct SharedBlackboard {
    /// Creator of the Blackboard.
    creator: ConstString,
    /// Hierarchy of this shared reference.
    path: ConstString,
    /// Shared reference to the [`Blackboard`]
    blackboard: Arc<RwLock<Blackboard>>,
}

impl Deref for SharedBlackboard {
    type Target = Arc<RwLock<Blackboard>>;

    fn deref(&self) -> &Self::Target {
        &self.blackboard
    }
}

impl DerefMut for SharedBlackboard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.blackboard
    }
}

impl BlackboardInterface for SharedBlackboard {
    fn contains(&self, key: &str) -> bool {
        // if it is a key starting with an '@' redirect to root bb
        if let Some(key_stripped) = key.strip_prefix('@') {
            return self.root().contains(key_stripped);
        }

        // Try in current Blackboard
        if self.read().content.read().contains(key) {
            return true;
        }

        // Try to find in parent hierarchy. We need to read the remapping info beforehand to avoid deadlocks.
        let (parent_key, has_remapping, autoremap) = self.get_parent_remapping_info(key);
        if has_remapping || autoremap {
            if let Some(parent) = &self.read().parent {
                return parent.contains(&parent_key);
            }
        }

        false
    }

    fn delete<T>(&mut self, key: &str) -> Result<T, Error>
    where
        T: Any + Clone + Debug + FromStr + ToString + Send + Sync,
    {
        // if it is a key starting with an '@' redirect to root bb
        if let Some(key_stripped) = key.strip_prefix('@') {
            return self.root().delete(key_stripped);
        }

        // Try to delete key in current Blackboard
        let a = self.write().content.write().delete::<T>(key);
        if a.is_ok() {
            return a;
        }

        // Try to find in parent hierarchy. We need to read the remapping info beforehand to avoid deadlocks.
        let (parent_key, has_remapping, autoremap) = self.get_parent_remapping_info(key);
        if has_remapping || autoremap {
            if let Some(parent) = &mut self.write().parent {
                return parent.delete(&parent_key);
            }
        }

        Err(Error::NotFound(key.into()))
    }

    fn get<T>(&self, key: &str) -> Result<T, Error>
    where
        T: Any + Clone + Debug + FromStr + ToString + Send + Sync,
    {
        // if it is a key starting with an '@' redirect to root bb
        if let Some(key_stripped) = key.strip_prefix('@') {
            return self.root().get(key_stripped);
        }

        // Try to find in current Blackboard
        let a = self.read().content.read().get::<T>(key);
        if a.is_ok() {
            return a;
        }

        // Try to find in parent hierarchy. We need to read the remapping info beforehand to avoid deadlocks.
        let (parent_key, has_remapping, autoremap) = self.get_parent_remapping_info(key);
        if has_remapping || autoremap {
            if let Some(parent) = &self.read().parent {
                return parent.get(&parent_key);
            }
        }

        Err(Error::NotFoundIn(parent_key, (&*self.path).into()))
    }

    fn get_sequence_id(&self, key: &str) -> Result<usize, Error> {
        // if it is a key starting with an '@' redirect to root bb
        if let Some(key_stripped) = key.strip_prefix('@') {
            return self.root().get_sequence_id(key_stripped);
        }

        // try to find key in current Blackboard
        if self.read().content.read().contains(key) {
            return self.read().content.read().get_sequence_id(key);
        }

        // Try to find in parent hierarchy. We need to read the remapping info beforehand to avoid deadlocks.
        let (parent_key, has_remapping, autoremap) = self.get_parent_remapping_info(key);
        if (has_remapping || autoremap) && self.read().content.read().contains(&parent_key) {
            return self.read().content.read().get_sequence_id(&parent_key);
        }

        Err(Error::NotFoundIn(parent_key, (&*self.path).into()))
    }

    fn get_entry(&self, key: &str) -> Option<Entry> {
        // if it is a key starting with an '@' redirect to root bb
        if let Some(key_stripped) = key.strip_prefix('@') {
            return self.root().get_entry(key_stripped);
        }

        // try to find key in current Blackboard
        let a = self.read().content.read().get_entry(key);
        if a.is_some() {
            return a;
        }

        // Try to find in parent hierarchy. We need to read the remapping info beforehand to avoid deadlocks.
        let (parent_key, has_remapping, autoremap) = self.get_parent_remapping_info(key);
        if has_remapping || autoremap {
            if let Some(parent) = &self.read().parent {
                return parent.get_entry(&parent_key);
            }
        }

        None
    }

    fn set<T>(&mut self, key: &str, value: T) -> Result<Option<T>, Error>
    where
        T: Any + Clone + Debug + FromStr + ToString + Send + Sync,
    {
        // if it is a key starting with an '@' redirect to root bb
        if let Some(key_stripped) = key.strip_prefix('@') {
            return self.root().set(key_stripped, value);
        }

        // Try to find key in current Blackboard
        let a = self.read().content.read().get::<T>(key);
        if a.is_ok() {
            return self.read().content.write().set(key, value);
        }

        // Try to find in parent hierarchy. We need to read the remapping info beforehand to avoid deadlocks.
        let (parent_key, has_remapping, autoremap) = self.get_parent_remapping_info(key);
        if has_remapping || autoremap {
            if let Some(parent) = &mut self.write().parent {
                return parent.set(&parent_key, value);
            }
        }

        // If it is not remapped to parent, set it in current `Blackboard`
        self.read().content.write().set(key, value)
    }
}

impl Environment for SharedBlackboard {
    fn define_env(&mut self, key: &str, value: ScriptingValue) -> Result<(), ScriptingError> {
        // if it is a key starting with an '@' redirect to root bb
        if let Some(key_stripped) = key.strip_prefix('@') {
            return self.root().define_env(key_stripped, value);
        }

        // Try to find key in current Blackboard
        let a = self.read().content.read().get::<ScriptingValue>(key);
        if a.is_ok() {
            return self.read().content.write().define_env(key, value);
        }

        // Try to find in parent hierarchy. Again we need to read the remapping info beforehand to avoid deadlocks.
        let (parent_key, has_remapping, autoremap) = self.get_parent_remapping_info(key);
        if has_remapping || autoremap {
            if let Some(parent) = &mut self.write().parent {
                return parent.define_env(&parent_key, value);
            }
        }

        // if it is not remapped to parent, set it in current `Blackboard`
        self.read().content.write().define_env(key, value)
    }

    fn get_env(&self, key: &str) -> Result<ScriptingValue, ScriptingError> {
        // if it is a key starting with an '@' redirect to root bb
        if let Some(key_stripped) = key.strip_prefix('@') {
            return self.root().get_env(key_stripped);
        }

        // Try to find key in current Blackboard
        let a = self.read().content.read().get_env(key);
        if let Ok(val) = a {
            return Ok(val);
        }

        // Try to find in parent hierarchy. Again we need to read the remapping info beforehand to avoid deadlocks.
        let (parent_key, has_remapping, autoremap) = self.get_parent_remapping_info(key);
        if has_remapping || autoremap {
            if let Some(parent) = &self.read().parent {
                return parent.get_env(&parent_key);
            }
        }

        Err(ScriptingError::GlobalNotDefined(key.into()))
    }

    fn set_env(&mut self, key: &str, value: ScriptingValue) -> Result<(), ScriptingError> {
        // if it is a key starting with an '@' redirect to root bb
        if let Some(key_stripped) = key.strip_prefix('@') {
            return self.root().set_env(key_stripped, value);
        }

        // Try to find key in current Blackboard
        let a = self.read().content.read().get::<ScriptingValue>(key);
        if a.is_ok() {
            return self.read().content.write().set_env(key, value);
        }

        // Try to find in parent hierarchy. Again we need to read the remapping info beforehand to avoid deadlocks.
        let (parent_key, has_remapping, autoremap) = self.get_parent_remapping_info(key);
        if has_remapping || autoremap {
            if let Some(parent) = &mut self.write().parent {
                return parent.set_env(&parent_key, value);
            }
        }

        Err(ScriptingError::GlobalNotDefined(key.into()))
    }
}

impl SharedBlackboard {
    /// Create a `SharedBlackboard` with remappings and an initial path.
    #[must_use]
    pub fn new(creator: &str) -> Self {
        let node = Blackboard::new();
        Self {
            creator: creator.into(),
            path: creator.into(),
            blackboard: Arc::new(RwLock::new(node)),
        }
    }

    /// Create a `SharedBlackboard` remappings.
    #[must_use]
    pub fn with(creator: &str, remappings: ConstPortRemappings) -> Self {
        Self {
            creator: creator.into(),
            path: creator.into(),
            blackboard: Arc::new(RwLock::new(Blackboard::with(remappings))),
        }
    }

    /// Create a `SharedBlackboard` with parent.
    #[must_use]
    pub fn with_parent(
        creator: &str,
        parent: Self,
        remappings: ConstPortRemappings,
        autoremap: bool,
    ) -> Self {
        let path = String::from(&*parent.path) + "/" + creator;
        Self {
            creator: creator.into(),
            path: path.into(),
            blackboard: Arc::new(RwLock::new(Blackboard::with_parent(
                parent, remappings, autoremap,
            ))),
        }
    }

    /// Get the creator
    #[must_use]
    pub const fn creator(&self) -> &ConstString {
        &self.creator
    }

    /// Print the content of the `SharedBlackboard` for debugging purpose.
    #[cfg(feature = "std")]
    pub fn debug_message(&self) {
        std::println!("{self:?}");
    }

    /// Read needed remapping information to parent.
    fn get_parent_remapping_info(&self, key: &str) -> (ConstString, bool, bool) {
        let guard = self.read();
        let (remapped_key, has_remapping) = guard.remappings_to_parent.as_ref().map_or_else(
            || (key.into(), false),
            |remappings| {
                let (remapped_key, has_remapping) = remappings
                    .find(&key.into())
                    .map_or_else(|| (key.into(), false), |remapped| (remapped, true));
                (strip_key(&remapped_key).into(), has_remapping)
            },
        );
        let autoremap = guard.autoremap_to_parent;
        drop(guard);
        (remapped_key, has_remapping, autoremap)
    }

    /// function to get access to the root [`BlackboardNode`]
    /// of a [`Blackboard`] tree in a recursive way.
    fn root(&self) -> Self {
        self.blackboard
            .read()
            .parent
            .as_ref()
            .map_or_else(|| self.clone(), Self::root)
    }

    /// Add or change the parent of a [`SharedBlackboard`].
    pub fn set_parent(&self, parent: Self) {
        self.write().parent = Some(parent);
    }

    #[must_use]
    pub(crate) fn remappings(&self) -> Option<ConstPortRemappings> {
        self.blackboard.read().remappings_to_parent.clone()
    }
}
// endregion:   --- SharedBlackboard
