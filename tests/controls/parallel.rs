// Copyright © 2025 Stephan Kunz

//! Tests the [`Parallel`] behavior

extern crate alloc;

use behaviortree::behavior::BehaviorState::*;
use behaviortree::behavior::action::ChangeStateAfter;
use behaviortree::prelude::*;

use rstest::rstest;

const PARALLEL: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Parallel name="root_parallel" success_count="1" failure_count="1">
			<Action ID="Action" name="action1"/>
			<Action ID="Action" name="action2"/>
			<Action	ID="Action" name="action3"/>
		</Parallel>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn parallel_raw() -> Result<(), Error> {
	fn set_values(
		tree: &mut BehaviorTree,
		action1_state: BehaviorState,
		action2_state: BehaviorState,
		action3_state: BehaviorState,
	) {
		for behavior in tree.iter_mut() {
			if behavior.name().as_ref() == "action1" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<ChangeStateAfter>()
				{
					behavior.set_final_state(action1_state);
				}
			}
			if behavior.name().as_ref() == "action2" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<ChangeStateAfter>()
				{
					behavior.set_final_state(action2_state);
				}
			}
			if behavior.name().as_ref() == "action3" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<ChangeStateAfter>()
				{
					behavior.set_final_state(action3_state);
				}
			}
		}
	}

	let mut factory = BehaviorTreeFactory::with_core_behaviors()?;
	register_behavior!(
		factory,
		ChangeStateAfter,
		"Action",
		BehaviorState::Running,
		BehaviorState::Failure,
		0
	)?;
	let mut tree = factory.create_from_text(PARALLEL)?;
	drop(factory);

	// case 1
	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	// case 2
	tree.reset()?;
	set_values(
		&mut tree,
		BehaviorState::Success,
		BehaviorState::Failure,
		BehaviorState::Failure,
	);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	// case 3
	tree.reset()?;
	set_values(
		&mut tree,
		BehaviorState::Success,
		BehaviorState::Success,
		BehaviorState::Success,
	);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	// case 4
	tree.reset()?;
	set_values(
		&mut tree,
		BehaviorState::Failure,
		BehaviorState::Success,
		BehaviorState::Success,
	);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);

	Ok(())
}

const TREE_DEFINITION: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Parallel name="simple_parallel">
			<Behavior1	name="step1"/>
			<Behavior2	name="step2"/>
			<Behavior3	name="step3"/>
		</Parallel>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Failure, Failure, Running, Running)]
#[case(Failure, Failure, Failure, Success)]
#[case(Success, Success, Running, Running)]
#[case(Success, Failure, Success, Success)]
#[case(Success, Success, Failure, Success)]
#[case(Success, Running, Failure, Running)]
#[case(Skipped, Skipped, Success, Success)]
#[case(Skipped, Skipped, Skipped, Skipped)]
#[case(Skipped, Skipped, Running, Running)]
#[case(Skipped, Skipped, Failure, Success)]
#[case(Success, Skipped, Success, Success)]
#[case(Success, Success, Success, Success)]
async fn simple_parallel(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] input3: BehaviorState,
	#[case] expected: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::default();
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
#[case(Success, Idle, Idle)]
#[case(Running, Idle, Idle)]
#[case(Success, Running, Idle)]
#[case(Failure, Running, Idle)]
#[case(Failure, Success, Idle)]
async fn simple_parallel_errors(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] input3: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::default();
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
#[case(Success, Failure, Running, Running, Success, Running)]
#[case(Failure, Success, Running, Running, Success, Running)]
async fn simple_parallel_reactiveness1(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] expected1: BehaviorState,
	#[case] expected2: BehaviorState,
	#[case] expected3: BehaviorState,
	#[case] expected4: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::default();
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
#[case(Failure, Success, Running, Running, Success, Running)]
async fn simple_parallel_reactiveness2(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] expected1: BehaviorState,
	#[case] expected2: BehaviorState,
	#[case] expected3: BehaviorState,
	#[case] expected4: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::default();
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
