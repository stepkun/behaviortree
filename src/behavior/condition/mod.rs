// Copyright © 2025 Stephan Kunz

//! Condition behavior library
//!

mod script_condition;
mod was_entry_updated;

// flatten
pub use script_condition::ScriptCondition;
pub use was_entry_updated::WasEntryUpdated;
