// Copyright © 2025 Stephan Kunz

//! This test implements the first tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev)
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-basics/tutorial_01_first_tree)
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t01_build_your_first_tree.cpp)
//!

extern crate alloc;
mod common;

use behaviortree::{
    behavior::{BehaviorKind, BehaviorState},
    factory::BehaviorTreeFactory,
    register_behavior,
};
use common::test_data::{ApproachObject, GripperInterface, check_battery};

/// This definition uses explicit node ID's
const XML_EXPLICIT: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Control ID="Sequence" name="root_sequence">
			<Condition ID="CheckBattery"	name="battery_ok"/>
			<Action ID="OpenGripper"		name="open_gripper"/>
			<Action ID="ApproachObject"		name="approach_object"/>
			<Action ID="CloseGripper"		name="close_gripper"/>
		</Control>
	</BehaviorTree>
</root>
"#;

async fn example() -> anyhow::Result<BehaviorState> {
    let mut factory = BehaviorTreeFactory::with_groot2_behaviors()?;

    // The recommended way to create a Behavior is through inheritance/composition.
    // Even if it requires more boilerplate, it allows you to use more functionalities
    // like ports (we will discuss this in future tutorials).
    register_behavior!(factory, ApproachObject, "ApproachObject")?;

    // Registering a SimpleAction/SimpleCondition using a function pointer.
    register_behavior!(
        factory,
        check_battery,
        "CheckBattery",
        BehaviorKind::Condition
    )?;

    // You can also create SimpleAction/SimpleCondition using methods of a struct.
    register_behavior!(
        factory,
        GripperInterface::default(),
        open,
        "OpenGripper",
        BehaviorKind::Action,
        close,
        "CloseGripper",
        BehaviorKind::Action,
    )?;

    let mut tree = factory.create_from_text(XML_EXPLICIT)?;
    // dropping the factory to free memory
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
    async fn t01_build_your_first_tree_explicit() -> anyhow::Result<()> {
        let result = example().await?;
        assert_eq!(result, BehaviorState::Success);
        Ok(())
    }
}
