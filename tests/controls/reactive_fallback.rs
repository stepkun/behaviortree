// Copyright © 2025 Stephan Kunz

//! Tests the [`ReactiveFallback`] behavior

extern crate alloc;

use behaviortree::{
	behavior::{BehaviorState::*, ChangeStateAfter},
	prelude::*,
};
use rstest::rstest;

const REACTIVE_FALLBACK: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<ReactiveFallback name="root_fallback">
			<Condition ID="Condition" name="condition1"/>
			<Condition ID="Condition" name="condition2"/>
			<Action	ID="Action" name="action"/>
		</ReactiveFallback>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn reactive_fallback_raw() -> Result<(), Error> {
	fn set_values(
		tree: &mut BehaviorTree,
		condition1_state: BehaviorState,
		condition2_state: BehaviorState,
		action_state: BehaviorState,
	) {
		for behavior in tree.iter_mut() {
			if behavior.name().as_ref() == "condition1" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<ChangeStateAfter>()
				{
					behavior.set_final_state(condition1_state);
				}
			}
			if behavior.name().as_ref() == "condition2" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<ChangeStateAfter>()
				{
					behavior.set_final_state(condition2_state);
				}
			}
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
		"Condition",
		BehaviorState::Running,
		BehaviorState::Failure,
		0
	)?;
	register_behavior!(
		factory,
		ChangeStateAfter,
		"Action",
		BehaviorState::Running,
		BehaviorState::Failure,
		0
	)?;
	let mut tree = factory.create_from_text(REACTIVE_FALLBACK)?;
	drop(factory);

	// case 1
	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	// case 2
	set_values(
		&mut tree,
		BehaviorState::Failure,
		BehaviorState::Failure,
		BehaviorState::Success,
	);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	// case 3
	set_values(
		&mut tree,
		BehaviorState::Failure,
		BehaviorState::Failure,
		BehaviorState::Failure,
	);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	set_values(
		&mut tree,
		BehaviorState::Success,
		BehaviorState::Failure,
		BehaviorState::Failure,
	);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);

	Ok(())
}

const TREE_DEFINITION: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<ReactiveFallback name="reactive_fallback">
			<Behavior1	name="step1"/>
			<Behavior2	name="step2"/>
			<Behavior3	name="step3"/>
		</ReactiveFallback>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Running, Idle, Idle, Running)]
#[case(Failure, Running, Idle, Running)]
#[case(Failure, Failure, Running, Running)]
#[case(Failure, Failure, Failure, Failure)]
#[case(Failure, Failure, Success, Success)]
#[case(Success, Idle, Idle, Success)]
#[case(Failure, Success, Idle, Success)]
#[case(Success, Running, Failure, Success)]
#[case(Skipped, Skipped, Success, Success)]
#[case(Skipped, Skipped, Skipped, Skipped)]
#[case(Skipped, Skipped, Running, Running)]
#[case(Skipped, Skipped, Failure, Failure)]
#[case(Success, Failure, Failure, Success)]
async fn reactive_fallback(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] input3: BehaviorState,
	#[case] expected: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input1, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", BehaviorState::Running, input2, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior3", BehaviorState::Running, input3, 0)?;

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
#[case(Idle, Idle, Idle)]
#[case(Idle, Success, Idle)]
#[case(Idle, Failure, Idle)]
#[case(Idle, Running, Idle)]
#[case(Idle, Skipped, Idle)]
#[case(Skipped, Skipped, Idle)]
async fn reactive_fallback_errors(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] input3: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input1, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", BehaviorState::Running, input2, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior3", BehaviorState::Running, input3, 0)?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let result = tree.tick_once().await;
	assert!(result.is_err());
	Ok(())
}

#[tokio::test]
#[rstest]
#[case(Success, Failure, Success, Success, Success, Success)]
#[case(Failure, Success, Running, Success, Running, Success)]
async fn reactive_fallback_reactiveness1(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] expected1: BehaviorState,
	#[case] expected2: BehaviorState,
	#[case] expected3: BehaviorState,
	#[case] expected4: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", input1, input2, 1)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", input1, input2, 2)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior3", input1, input2, 3)?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let mut result = tree.tick_once().await?;
	assert_eq!(result, expected1);
	result = tree.tick_once().await?;
	assert_eq!(result, expected2);
	result = tree.tick_once().await?;
	assert_eq!(result, expected3);
	result = tree.tick_once().await?;
	assert_eq!(result, expected4);

	tree.reset()?;

	result = tree.tick_once().await?;
	assert_eq!(result, expected1);
	result = tree.tick_once().await?;
	assert_eq!(result, expected2);
	result = tree.tick_once().await?;
	assert_eq!(result, expected3);
	result = tree.tick_once().await?;
	assert_eq!(result, expected4);

	Ok(())
}

#[tokio::test]
#[rstest]
#[case(Success, Failure, Running, Running, Success, Running)]
#[case(Failure, Success, Running, Running, Running, Success)]
async fn reactive_fallback_reactiveness2(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] expected1: BehaviorState,
	#[case] expected2: BehaviorState,
	#[case] expected3: BehaviorState,
	#[case] expected4: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", input1, input2, 3)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", input1, input2, 2)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior3", input1, input2, 1)?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let mut result = tree.tick_once().await?;
	assert_eq!(result, expected1);
	result = tree.tick_once().await?;
	assert_eq!(result, expected2);
	result = tree.tick_once().await?;
	assert_eq!(result, expected3);
	result = tree.tick_once().await?;
	assert_eq!(result, expected4);

	tree.reset()?;

	result = tree.tick_once().await?;
	assert_eq!(result, expected1);
	result = tree.tick_once().await?;
	assert_eq!(result, expected2);
	result = tree.tick_once().await?;
	assert_eq!(result, expected3);
	result = tree.tick_once().await?;
	assert_eq!(result, expected4);

	Ok(())
}
