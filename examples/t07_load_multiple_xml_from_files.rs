// Copyright © 2025 Stephan Kunz
//! Implements the seventh tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev)
//! using the include statement in xml file.
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-basics/tutorial_07_multiple_xml).
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t07_load_multiple_xml.cpp).

mod common;

use behaviortree::prelude::*;
use common::test_data::SaySomething;

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_core_behaviors()?;

	register_behavior!(factory, SaySomething, "SaySomething")?;

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

#[tokio::main]
async fn main() -> Result<(), Error> {
	example().await?;
	Ok(())
}

#[cfg(test)]
mod test {
	use super::*;

	#[tokio::test]
	async fn t07_load_multiple_xml_from_files() -> Result<(), Error> {
		let result = example().await?;
		assert_eq!(result, BehaviorState::Success);
		Ok(())
	}
}
