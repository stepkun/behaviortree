// Copyright © 2025 Stephan Kunz

//! Tests the [`Inverter`] decorator

extern crate alloc;

use behaviortree::{
	behavior::{BehaviorState::*, ChangeStateAfter},
	prelude::*,
};
use rstest::rstest;

const INVERTER: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Inverter name="root_inverter">
			<Action	ID="Action" name="action"/>
		</Inverter>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn inverter_raw() -> Result<(), Error> {
	fn set_values(tree: &mut BehaviorTree, action_state: BehaviorState) {
		for behavior in tree.iter_mut() {
			if behavior.name().as_ref() == "action" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<ChangeStateAfter>()
				{
					behavior.set_final_state(action_state);
				}
			}
		}
	}

	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(
		factory,
		ChangeStateAfter,
		"Action",
		BehaviorState::Running,
		BehaviorState::Failure,
		0
	)?;

	let mut tree = factory.create_from_text(INVERTER)?;
	drop(factory);

	// case 1
	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	// case 2
	set_values(&mut tree, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	// case 3
	set_values(&mut tree, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	// case 4
	set_values(&mut tree, BehaviorState::Skipped);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Skipped);

	Ok(())
}

const TREE_DEFINITION: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Inverter name="inverter">
			<Behavior1	name="child"/>
		</Inverter>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Running, Running)]
#[case(Skipped, Skipped)]
#[case(Failure, Success)]
#[case(Success, Failure)]
async fn inverter(#[case] input: BehaviorState, #[case] expected: BehaviorState) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input, 0)?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let mut result = tree.tick_once().await?;
	assert_eq!(result, expected);
	result = tree.tick_once().await?;
	assert_eq!(result, expected);

	tree.reset()?;

	result = tree.tick_once().await?;
	assert_eq!(result, expected);
	result = tree.tick_once().await?;
	assert_eq!(result, expected);

	Ok(())
}

#[tokio::test]
#[rstest]
#[case(Idle)]
async fn inverter_errors(#[case] input: BehaviorState) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input, 0)?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let result = tree.tick_once().await;
	assert!(result.is_err());
	Ok(())
}
