// Copyright © 2025 Stephan Kunz
//! Built-in `Condition` behaviors of [`behaviortree`](crate).

#[cfg(feature = "script_condition")]
mod script_condition;
#[cfg(feature = "was_entry_updated")]
mod was_entry_updated;

// flatten
#[cfg(feature = "script_condition")]
pub use script_condition::ScriptCondition;
#[cfg(feature = "was_entry_updated")]
pub use was_entry_updated::WasEntryUpdated;
