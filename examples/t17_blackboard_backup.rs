// Copyright © 2025 Stephan Kunz
#![allow(unused)]

//! This test implements the seventeenth tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev)
//!
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t17_blackboard_backup.cpp)
//!

// //! [tutorial:](https://www.behaviortree.dev/docs/tutorial-advanced/tutorial_17_XXX)

extern crate alloc;

use std::{
    fmt::{Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};

use behaviortree::{
    Behavior, SharedRuntime,
    behavior::{
        BehaviorError, BehaviorInstance, BehaviorKind, BehaviorResult, BehaviorState,
        BehaviorStatic,
    },
    blackboard::{BlackboardInterface, SharedBlackboard},
    factory::BehaviorTreeFactory,
    input_port, output_port,
    port::PortList,
    port_list,
    tree::BehaviorTreeElementList,
};

const XML: &str = r#"
<root BTCPP_format="4">
  	<BehaviorTree ID="MainTree">
		<AlwaysSuccess/>
  	</BehaviorTree>
</root>
"#;

// @TODO: implement
async fn example() -> anyhow::Result<BehaviorState> {
    let mut factory = BehaviorTreeFactory::with_groot2_behaviors()?;

    // register_behavior!(factory, SaySomething, "SaySomething")?;

    factory.register_behavior_tree_from_text(XML)?;

    let mut tree = factory.create_tree("MainTree")?;
    drop(factory);

    let result = tree.tick_while_running().await?;
    Ok(result)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    example().await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn t17_blackboard_backup() -> anyhow::Result<()> {
        let result = example().await?;
        assert_eq!(result, BehaviorState::Success);
        Ok(())
    }
}
