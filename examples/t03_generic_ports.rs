// Copyright © 2025 Stephan Kunz

//! This test implements the third tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev)
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-basics/tutorial_03_generic_ports)
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t03_generic_ports.cpp)
//!

#[doc(hidden)]
extern crate alloc;
mod common;

use behaviortree::prelude::*;
use common::test_data::{CalculateGoal, PrintTarget};

const XML: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Sequence name="root">
            <CalculateGoal   goal="{GoalPosition}" />
            <PrintTarget     target="{GoalPosition}" />
            <Script          code="OtherGoal:='-1;3'" />
            <PrintTarget     target="{OtherGoal}" />
		</Sequence>
	</BehaviorTree>
</root>
"#;

async fn example() -> BehaviorTreeResult {
    let mut factory = BehaviorTreeFactory::with_groot2_behaviors()?;

    register_behavior!(factory, CalculateGoal, "CalculateGoal")?;
    register_behavior!(factory, PrintTarget, "PrintTarget")?;

    let mut tree = factory.create_from_text(XML)?;
    drop(factory);

    let result = tree.tick_while_running().await?;
    Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    example().await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn t03_generic_ports() -> Result<(), Error> {
        let result = example().await?;
        assert_eq!(result, BehaviorState::Success);
        Ok(())
    }
}
