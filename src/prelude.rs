// Copyright © 2025 Stephan Kunz

//! Most commonly used interfaces of `behaviortree`.
//!
//! Typically it is sufficient to include the prelude with
//!
//! ```use behaviortree::prelude::*;```

// to avoid adding these crates to dependencies
pub extern crate alloc;
pub extern crate tinyscript;

// re-exports
pub use alloc::{
	boxed::Box,
	str::FromStr,
	string::{String, ToString},
	vec::Vec,
};
pub use behaviortree_derive::{Action, Condition, Control, Decorator};
// databoard
pub use databoard::{Databoard, Remappings};
// tinyscript
pub use tinyscript::{ScriptEnum, SharedRuntime};

// public exports
// literals
pub use crate::EMPTY_STR;
// error handling
pub use crate::error::{BehaviorTreeResult, Error};
// behavior macros
pub use crate::{register_behavior, register_scripting_enum};
// port macros
pub use crate::{inout_port, input_port, output_port, port_list};
// behavior
pub use crate::behavior::{
	BehaviorError, BehaviorKind, BehaviorResult, BehaviorState, behavior_data::BehaviorData,
	behavior_description::BehaviorDescription,
};
// behavior traits
pub use crate::behavior::{Behavior, BehaviorExecution};
// factory
pub use crate::factory::BehaviorTreeFactory;
// port
pub use crate::port::PortList;
// tree
pub use crate::tree::{BehaviorTree, BehaviorTreeElementList};
