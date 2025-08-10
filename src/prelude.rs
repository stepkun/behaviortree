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
pub use behaviortree_derive::Behavior;
pub use tinyscript::SharedRuntime;
pub use tinyscript::ScriptEnum;

// exports
// error handling
pub use crate::error::{BehaviorTreeResult, Error};
// behavior macros
pub use crate::{register_behavior, register_scripting_enum};
// port macros
pub use crate::{port_list, inout_port, input_port, output_port};
// behavior
pub use crate::behavior::{BehaviorData, BehaviorDescription, BehaviorError, BehaviorResult, BehaviorState};
pub use crate::behavior::BehaviorKind;  // maybe this one will become obsolete
// behavior traits
pub use crate::behavior::{BehaviorInstance, BehaviorStatic};
// blackboard
pub use crate::blackboard::SharedBlackboard;
// blackboard traits
pub use crate::blackboard::BlackboardInterface;
// factory
pub use crate::factory::BehaviorTreeFactory;
// port
pub use crate::port::PortList;
// tree
pub use crate::tree::{BehaviorTree, ConstBehaviorTreeElementList};
