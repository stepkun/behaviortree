// Copyright © 2025 Stephan Kunz

//! This test implements the first tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev)
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-basics/tutorial_01_first_tree)
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t01_build_your_first_tree.cpp)
//!

extern crate alloc;
mod common;

use behaviortree::prelude::*;
use common::test_data::{ApproachObject, GripperInterface, check_battery};

/// This definition uses implicit node ID's
const XML: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Sequence name="root_sequence">
			<CheckBattery	name="battery_ok"/>
			<OpenGripper	name="open_gripper"/>
			<ApproachObject	name="approach_object"/>
			<CloseGripper	name="close_gripper"/>
		</Sequence>
	</BehaviorTree>
</root>
"#;

async fn example() -> BehaviorTreeResult {
    let mut factory = BehaviorTreeFactory::default();

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

    // Trees are created at run-time, but only once at the beginning.
    // The currently supported format is XML.
    // IMPORTANT: When the object "tree" goes out of scope, all the tree components are destroyed
    let mut tree = factory.create_from_text(XML)?;
    // dropping the factory to free memory
    drop(factory);

    // To "execute" a Tree you need to "tick" it.
    // The tick is propagated to the children based on the logic of the tree.
    // In this case, the entire sequence is executed, because all the children
    // of the Sequence return SUCCESS.
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
    async fn t01_build_your_first_tree() -> Result<(), Error> {
        let result = example().await?;
        assert_eq!(result, BehaviorState::Success);
        Ok(())
    }
}
