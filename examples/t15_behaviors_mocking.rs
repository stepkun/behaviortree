// Copyright © 2025 Stephan Kunz
//! Implements the fifteenth tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev).
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-advanced/tutorial_15_replace_rules).
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t15_nodes_mocking.cpp).

mod common;

use crate::common::test_data::SaySomething;
use behaviortree::prelude::*;

const XML: &str = r#"
<root BTCPP_format="4">
  	<BehaviorTree ID="MainTree">
		<Sequence>
			<SaySomething name="talk" message="hello world"/>

			<SubTree ID="MySub" name="mysub"/>

			<Script name="set_message" code="msg:= 'the original message' "/>
			<SaySomething message="{msg}"/>

			<Sequence name="counting">
				<SaySomething message="1"/>
				<SaySomething message="2"/>
				<SaySomething message="3"/>
			</Sequence>
		</Sequence>
  	</BehaviorTree>

	<BehaviorTree ID="MySub">
		<Sequence>
		<AlwaysSuccess name="action_subA"/>
		<AlwaysSuccess name="action_subB"/>
		</Sequence>
	</BehaviorTree>
</root>
"#;

// @TODO: implement
async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_groot2_behaviors()?;
	factory.register_test_behaviors()?;
	register_behavior!(factory, SaySomething, "SaySomething")?;

	factory.register_behavior_tree_from_text(XML)?;

	let mut tree = factory.create_tree("MainTree")?;
	drop(factory);

	// as a reminder, let's print the full names of all the nodes
	println!("----- Nodes fullPath() -------");
	for behavior in tree.iter() {
		println!("{}", behavior.full_path());
	}

	println!("\n------ Output (original) ------");
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
	#[ignore = "not yet implemented"]
	async fn t15_behaviors_mocking() -> Result<(), Error> {
		let result = example().await?;
		assert_eq!(result, BehaviorState::Success);
		Ok(())
	}
}
