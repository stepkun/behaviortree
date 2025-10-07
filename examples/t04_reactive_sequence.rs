// Copyright © 2025 Stephan Kunz
//! Implements the fourth tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev).
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-basics/tutorial_04_sequence).
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t04_reactive_sequence.cpp).

#[path = "./common/test_data.rs"]
mod test_data;

use behaviortree::prelude::*;
use std::time::Duration;
use test_data::{MoveBaseAction, SaySomething, check_battery};

const XML: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
        <Sequence name="std root sequence">
            <BatteryOK/>
            <SaySomething   message="mission started..." />
            <MoveBase       goal="1;2;3"/>
            <SaySomething   message="mission completed!" />
        </Sequence>
	</BehaviorTree>
</root>
"#;

const XML_REACTIVE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="ReactiveMainTree">
	<BehaviorTree ID="ReactiveMainTree">
		<ReactiveSequence name="reactive root sequence">
            <BatteryOK/>
            <Sequence name = "inner std sequence">
                <SaySomething   message="mission started..." />
                <MoveBase       goal="1;2;3"/>
                <SaySomething   message="mission completed!" />
            </Sequence>
		</ReactiveSequence>
	</BehaviorTree>
</root>
"#;

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::new()?;

	register_simple_behavior!(factory, check_battery, "BatteryOK", BehaviorKind::Condition)?;
	register_behavior!(factory, MoveBaseAction, "MoveBase")?;
	register_behavior!(factory, SaySomething, "SaySomething")?;

	let mut tree = factory.create_from_text(XML)?;
	let mut reactive_tree = factory.create_from_text(XML_REACTIVE)?;
	drop(factory);

	// run the BT using own loop with sleep to avoid busy loop
	println!("=> Running BT with std sequence:");
	let mut result = tree.tick_once().await?;
	while result == BehaviorState::Running {
		tokio::time::sleep(Duration::from_millis(100)).await;
		result = tree.tick_once().await?;
	}

	// run the reactive BT using own loop with sleep to avoid busy loop
	println!("\n\n=> Running BT with reactive sequence:");
	let mut result = reactive_tree.tick_once().await?;
	while result == BehaviorState::Running {
		tokio::time::sleep(Duration::from_millis(100)).await;
		result = reactive_tree.tick_once().await?;
	}
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
	async fn t04_reactive_sequence() -> Result<(), Error> {
		let result = example().await?;
		assert_eq!(result, BehaviorState::Success);
		Ok(())
	}
}
