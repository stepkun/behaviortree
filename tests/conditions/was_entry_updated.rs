// Copyright © 2025 Stephan Kunz

//! Tests the [`WasEntryUpdated`] condition

extern crate alloc;

use behaviortree::{behavior::condition::WasEntryUpdated, prelude::*};

const TREE_DEFINITION: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<WasEntryUpdated name="was_entry_updated" entry="test" />
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn was_entry_updated() -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, WasEntryUpdated, "WasEntryUpdated")?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	tree.blackboard().set("test", 1)?;
	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	tree.blackboard().set("test", 2)?;
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);

	tree.blackboard().delete::<i32>("test")?;
	tree.reset()?;

	tree.blackboard().set("test", 1)?;
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	tree.blackboard().set("test", 2)?;
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);

	Ok(())
}

#[tokio::test]
async fn was_entry_updated_error() -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, WasEntryUpdated, "WasEntryUpdated")?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let mut result = tree.tick_once().await;
	assert!(result.is_err());
	result = tree.tick_once().await;
	assert!(result.is_err());

	tree.reset()?;

	assert!(result.is_err());
	result = tree.tick_once().await;
	assert!(result.is_err());

	Ok(())
}
