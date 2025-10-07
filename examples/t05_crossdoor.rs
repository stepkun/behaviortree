// Copyright © 2025 Stephan Kunz
//! Implements the fifth tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev).
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-basics/tutorial_05_subtrees).
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t01_build_your_first_tree.cpp).

#[path = "./common/cross_door.rs"]
mod cross_door;

use behaviortree::prelude::*;
use cross_door::CrossDoor;

const XML: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="CrossDoor">
        <Sequence>
            <Fallback>
                <Inverter>
                    <IsDoorClosed/>
                </Inverter>
                <SubTree ID="DoorClosed"/>
            </Fallback>
            <PassThroughDoor/>
        </Sequence>
	</BehaviorTree>

    <BehaviorTree ID="DoorClosed">
        <Fallback>
            <OpenDoor/>
            <RetryUntilSuccessful num_attempts="5">
                <PickLock/>
            </RetryUntilSuccessful>
            <SmashDoor/>
        </Fallback>
    </BehaviorTree>
</root>
"#;

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::new()?;

	CrossDoor::register_behaviors(&mut factory)?;

	// In this example a single XML contains multiple <BehaviorTree>
	// To determine which one is the "main one", we should first register
	// the XML and then allocate a specific tree, using its ID
	factory.register_behavior_tree_from_text(XML)?;
	let mut tree = factory.create_tree("CrossDoor")?;
	drop(factory);

	// helper function to print the tree
	tree.print()?;

	// Tick multiple times, until either FAILURE of SUCCESS is returned
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
	async fn t05_crossdoor() -> Result<(), Error> {
		let result = example().await?;
		assert_eq!(result, BehaviorState::Success);
		Ok(())
	}
}
