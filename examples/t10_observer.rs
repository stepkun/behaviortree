// Copyright © 2025 Stephan Kunz
//! Implements the tenth tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev).
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-basics/tutorial_10_observer).
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t10_observer.cpp).

use behaviortree::{BehaviorTreeObserver, prelude::*};

const XML: &str = r#"
<root BTCPP_format="4">
    <BehaviorTree ID="MainTree">
        <Sequence>
            <Fallback>
                <AlwaysFailure name="failing_action"/>
                <SubTree ID="SubTreeA" name="mysub"/>
            </Fallback>
            <AlwaysSuccess name="last_action"/>
        </Sequence>
    </BehaviorTree>

    <BehaviorTree ID="SubTreeA">
        <Sequence>
            <AlwaysSuccess name="action_subA"/>
            <SubTree ID="SubTreeB" name="sub_nested"/>
            <SubTree ID="SubTreeB" />
        </Sequence>
    </BehaviorTree>

    <BehaviorTree ID="SubTreeB">
        <AlwaysSuccess name="action_subB"/>
    </BehaviorTree>

</root>
"#;

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_groot2_behaviors()?;

	factory.register_behavior_tree_from_text(XML)?;

	let mut tree = factory.create_tree("MainTree")?;
	drop(factory);

	// add the observer
	let observer = BehaviorTreeObserver::new(&mut tree);

	// print tree structure
	tree.print()?;
	println!();

	// Print the unique ID and the corresponding human readable path
	// Path is also expected to be unique.
	for node in tree.iter() {
		println!("{} <-> {}", node.uid(), node.data().description().path());
	}
	println!();

	let result = tree.tick_while_running().await?;

	// print statistics
	for item in tree.iter() {
		let stats = observer
			.get_statistics(item.uid())
			.expect("should be there");
		println!(
			"[{}]  T/S/F: {}/{}/{}",
			item.data().description().path(),
			stats.transitions_count,
			stats.success_count,
			stats.failure_count
		);
	}
	println!();

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
	async fn t10_observer() -> Result<(), Error> {
		let result = example().await?;
		assert_eq!(result, BehaviorState::Success);
		Ok(())
	}
}
