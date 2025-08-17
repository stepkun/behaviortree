// Copyright © 2025 Stephan Kunz

//! This test implements the seventh tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev)
//! using the include statement in xml file
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-basics/tutorial_07_multiple_xml)
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t07_load_multiple_xml.cpp)

mod common;

use behaviortree::prelude::*;
use common::test_data::SaySomething;

const XML_WITH_INCLUDE: &str = r#"
<root BTCPP_format="4">
    <include path="./subtree_A.xml" />
    <include path="./subtree_B.xml" />
    <BehaviorTree ID="MainTree">
        <Sequence>
            <SaySomething message="starting MainTree" />
            <SubTree ID="SubA"/>
            <SubTree ID="SubB"/>
        </Sequence>
    </BehaviorTree>
</root>
"#;

async fn example() -> BehaviorTreeResult {
	// set the necessary directory, currently only working if in project root directory
	// @TODO: find a better solution for this.
	let mut dir = std::env::current_dir()?;
	dir.push("examples");
	std::env::set_current_dir(dir)?;

	let mut factory = BehaviorTreeFactory::with_core_behaviors()?;

	register_behavior!(factory, SaySomething, "SaySomething")?;

	// Register the behavior tree definition, but do not instantiate the tree yet.
	factory.register_behavior_tree_from_text(XML_WITH_INCLUDE)?;

	//Check that the BTs have been registered correctly
	println!("Registered BehaviorTrees:");
	for bt_name in factory.registered_behavior_trees() {
		println!(" - {bt_name}");
	}

	// You can create the MainTree and the subtrees will be added automatically.
	let mut tree = factory.create_tree("MainTree")?;
	// ... and/or you can create only one of the subtrees
	let mut sub_tree_a = factory.create_tree("SubA")?;
	drop(factory);

	println!("----- MainTree tick ----");
	let result = tree.tick_while_running().await?;

	println!("----- SubA tick ----");
	sub_tree_a.tick_while_running().await?;

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
	#[ignore = "does not yet work"]
	async fn t07_load_multiple_xml_with_include() -> Result<(), Error> {
		let result = example().await?;
		assert_eq!(result, BehaviorState::Success);
		Ok(())
	}
}
