// Copyright © 2025 Stephan Kunz

//! Tests the [`EntryUpdated`] decorator

extern crate alloc;

use crate::decorators::utilities::ChangeStateAfter;
use behaviortree::{
	behavior::{BehaviorState::*, MockBehavior, MockBehaviorConfig, decorator::EntryUpdated},
	prelude::*,
};
use rstest::rstest;

const ENTRY_UPDATED: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<EntryUpdated name="entry_updated" entry="test">
			<Action	ID="Action" name="action"/>
		</EntryUpdated>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn entry_updated_raw() -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;

	EntryUpdated::register_with(&mut factory, "EntryUpdated", BehaviorState::Idle, true)?;
	MockBehavior::register_with(&mut factory, "Action", MockBehaviorConfig::new(BehaviorState::Success), true)?;

	let mut tree = factory.create_from_text(ENTRY_UPDATED)?;
	drop(factory);

	tree.blackboard().set("test", 1)?;
	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Idle);
	tree.blackboard().set("test", 2)?;
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Idle);
	for behavior in tree.iter_mut() {
		if behavior.name().as_ref() == "entry_updated" {
			if let Some(behavior) = behavior
				.behavior_mut()
				.as_any_mut()
				.downcast_mut::<EntryUpdated>()
			{
				behavior.initialize(BehaviorState::Failure);
			}
		}
	}
	tree.blackboard().set("test", 1)?;
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	tree.blackboard().set("test", 2)?;
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);

	Ok(())
}

const TREE_DEFINITION: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<EntryUpdated name="entry_updated" entry="test">
			<Behavior1	name="child"/>
		</EntryUpdated>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Running)]
#[case(Skipped)]
#[case(Failure)]
#[case(Success)]
async fn entry_updated(#[case] input: BehaviorState) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	ChangeStateAfter::register(&mut factory, "Behavior1", BehaviorState::Running, BehaviorState::Success, 0)?;

	EntryUpdated::register_with(&mut factory, "EntryUpdated", input, true)?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	tree.blackboard().set("test", 1)?;
	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, input);
	tree.blackboard().set("test", 2)?;
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);

	tree.blackboard().delete::<i32>("test")?;
	tree.reset()?;

	tree.blackboard().set("test", 1)?;
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, input);
	tree.blackboard().set("test", 2)?;
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);

	Ok(())
}

#[tokio::test]
#[rstest]
#[case(Idle)]
#[case(Running)]
#[case(Failure)]
#[case(Skipped)]
#[case(Success)]
async fn entry_updated_errors(#[case] input: BehaviorState) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	ChangeStateAfter::register(&mut factory, "Behavior1", BehaviorState::Running, BehaviorState::Success, 0)?;
	EntryUpdated::register_with(&mut factory, "EntryUpdated", input, true)?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let result = tree.tick_once().await;
	assert!(result.is_err());
	Ok(())
}
