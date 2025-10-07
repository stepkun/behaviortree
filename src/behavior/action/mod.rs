// Copyright © 2025 Stephan Kunz
//! Built-in `Action` behaviors of [`behaviortree`](crate).

#[cfg(feature = "pop_from_queue")]
mod pop_from_queue;
#[cfg(feature = "script")]
mod script;
#[cfg(feature = "set_blackboard")]
mod set_blackboard;
#[cfg(feature = "sleep")]
mod sleep;
#[cfg(feature = "unset_blackboard")]
mod unset_blackboard;

// flatten
#[cfg(feature = "pop_from_queue")]
pub use pop_from_queue::PopFromQueue;
#[cfg(feature = "script")]
pub use script::Script;
#[cfg(feature = "set_blackboard")]
pub use set_blackboard::SetBlackboard;
#[cfg(feature = "sleep")]
pub use sleep::Sleep;
#[cfg(feature = "unset_blackboard")]
pub use unset_blackboard::UnsetBlackboard;
