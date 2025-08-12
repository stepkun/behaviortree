// Copyright © 2024 Stephan Kunz

//! Most commonly used interface of `behaviortree`.
//!
//! Typically it is sufficient to include the prelude with
//!
//! ```use behaviortree::prelude::*;```

// to avoid adding these crates to dependencies
pub extern crate alloc;
pub extern crate tinyscript;

// re-exports
#[cfg(not(feature = "std"))]
pub use alloc::{
	boxed::Box,
	str::FromStr,
	string::{String, ToString},
	vec::Vec,
};
pub use behaviortree_derive::{Action, Condition, Control, Decorator};
pub use tinyscript::ScriptEnum;
pub use tinyscript::SharedRuntime;

// public exports
// error handling
pub use crate::error::{BehaviorTreeResult, Error};
// behavior macros
pub use crate::{register_behavior, register_scripting_enum};
// port macros
pub use crate::{inout_port, input_port, output_port, port_list};
// behavior
pub use crate::behavior::{BehaviorData, BehaviorDescription, BehaviorError, BehaviorKind, BehaviorResult, BehaviorState};
// behavior traits
pub use crate::behavior::{Behavior, BehaviorExecution, BehaviorInstance, BehaviorStatic};
// blackboard
pub use crate::blackboard::SharedBlackboard;
// blackboard traits
pub use crate::blackboard::BlackboardInterface;
// factory
pub use crate::factory::BehaviorTreeFactory;
// port
pub use crate::port::PortList;
// tree
pub use crate::tree::{tree::BehaviorTree, tree_element_list::ConstBehaviorTreeElementList};
