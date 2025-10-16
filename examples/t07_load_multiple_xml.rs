// Copyright © 2025 Stephan Kunz
//! Implements the seventh tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev).
//! It shows three ways to work with additional XML definitions:
//! - using separate xml definitions directly in the source file,
//! - load a tree definition from a file and
//! - include additional files via an 'include' tag.
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-basics/tutorial_07_multiple_xml).
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t07_load_multiple_xml.cpp).

#[path = "./common/test_data.rs"]
mod test_data;

use behaviortree::prelude::*;
use test_data::SaySomething;

const XML_MAIN: &str = r#"
<root BTCPP_format="4">
    <BehaviorTree ID="MainTree">
        <Sequence>
            <SaySomething message="starting MainTree" />
            <SubTree ID="SubA"/>
            <SubTree ID="SubB"/>
        </Sequence>
    </BehaviorTree>
</root>
"#;

const XML_SUB_A: &str = r#"
<root BTCPP_format="4">
    <BehaviorTree ID="SubA">
        <SaySomething message="Executing SubA" />
    </BehaviorTree>
</root>
"#;

const XML_SUB_B: &str = r#"
<root BTCPP_format="4">
    <BehaviorTree ID="SubB">
        <SaySomething message="Executing SubB" />
    </BehaviorTree>
</root>
"#;

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::new()?;

	SaySomething::register(&mut factory, "SaySomething")?;

	// Register the behavior tree definitions, but do not instantiate them yet.
	// Order is not important.
	factory.register_behavior_tree_from_text(XML_SUB_A)?;
	factory.register_behavior_tree_from_text(XML_SUB_B)?;
	factory.register_behavior_tree_from_text(XML_MAIN)?;

	// Check that the BTs have been registered correctly
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

async fn example_from_file() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::new()?;

	SaySomething::register(&mut factory, "SaySomething")?;

	// Load tree from files, but do not instantiate the tree yet.
	// The subdir 'examples' is necessary because typically this is started from workspace directory.
	factory.register_behavior_tree_from_file("./examples/maintree.xml")?;

	// Check that the BTs have been registered correctly
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

async fn example_with_include() -> BehaviorTreeResult {
	// set the necessary directory, only working if in project root directory
	let mut dir = std::env::current_dir()?;
	dir.push("examples");
	std::env::set_current_dir(dir)?;

	let mut factory = BehaviorTreeFactory::new()?;

	SaySomething::register(&mut factory, "SaySomething")?;

	// Register the behavior tree definition, but do not instantiate the tree yet.
	factory.register_behavior_tree_from_text(XML_WITH_INCLUDE)?;

	// Check that the BTs have been registered correctly
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
	example_from_file().await?;
	example_with_include().await?;
	Ok(())
}

#[cfg(test)]
mod test {
	use super::*;

	#[tokio::test]
	async fn t07_load_multiple_xml() -> Result<(), Error> {
		let result = example().await?;
		assert_eq!(result, BehaviorState::Success);
		let result = example_from_file().await?;
		assert_eq!(result, BehaviorState::Success);
		let result = example_with_include().await?;
		assert_eq!(result, BehaviorState::Success);
		Ok(())
	}
}
