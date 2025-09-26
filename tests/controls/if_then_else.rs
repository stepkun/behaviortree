// Copyright © 2025 Stephan Kunz

//! Tests the [`IfThenElse`] behavior

extern crate alloc;

use behaviortree::behavior::BehaviorState::*;
use behaviortree::behavior::{action::ChangeStateAfter, control::IfThenElse};
use behaviortree::prelude::*;

use rstest::rstest;

const IF_THEN_ELSE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<IfThenElse name="root_if_then_else">
			<Condition ID="Condition" name="if"/>
			<Action ID="Action" name="then"/>
			<Action	ID="Action" name="else"/>
		</IfThenElse>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn if_then_else_raw() -> Result<(), Error> {
	fn set_values(
		tree: &mut BehaviorTree,
		condition_state: BehaviorState,
		then_action_state: BehaviorState,
		else_action_state: BehaviorState,
	) {
		for behavior in tree.iter_mut() {
			if behavior.name().as_ref() == "if" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<ChangeStateAfter>()
				{
					behavior.set_final_state(condition_state);
				}
			}
			if behavior.name().as_ref() == "then" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<ChangeStateAfter>()
				{
					behavior.set_final_state(then_action_state);
				}
			}
			if behavior.name().as_ref() == "else" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<ChangeStateAfter>()
				{
					behavior.set_final_state(else_action_state);
				}
			}
		}
	}

	let mut factory = BehaviorTreeFactory::with_extended_behaviors()?;
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
	let mut tree = factory.create_from_text(IF_THEN_ELSE)?;
	drop(factory);

	// case 1
	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	// case 2
	set_values(&mut tree, BehaviorState::Success, BehaviorState::Failure, BehaviorState::Idle);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	// case 3
	set_values(&mut tree, BehaviorState::Success, BehaviorState::Success, BehaviorState::Idle);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	// case 4
	set_values(&mut tree, BehaviorState::Failure, BehaviorState::Idle, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	// case 4
	set_values(&mut tree, BehaviorState::Success, BehaviorState::Success, BehaviorState::Idle);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	set_values(&mut tree, BehaviorState::Failure, BehaviorState::Idle, BehaviorState::Failure);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);

	Ok(())
}

const TREE_DEFINITION: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<IfThenElse name="if_then_else">
			<Behavior1	name="if"/>
			<Behavior2	name="then"/>
			<Behavior3	name="else"/>
		</IfThenElse>
	</BehaviorTree>
</root>
"#;

const TREE_DEFINITION_2_CHILDREN: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<IfThenElse name="if_then_else">
			<Behavior1	name="if"/>
			<Behavior2	name="then"/>
		</IfThenElse>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Running, Idle, Idle, Running)]
#[case(Failure, Idle, Running, Running)]
#[case(Failure, Idle, Failure, Failure)]
#[case(Failure, Idle, Success, Success)]
#[case(Success, Running, Idle, Running)]
#[case(Success, Failure, Idle, Failure)]
#[case(Success, Success, Idle, Success)]
#[case(Skipped, Skipped, Skipped, Skipped)]
async fn if_then_else(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] input3: BehaviorState,
	#[case] expected: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input1, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", BehaviorState::Running, input2, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior3", BehaviorState::Running, input3, 0)?;
	register_behavior!(factory, IfThenElse, "IfThenElse")?;

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
#[case(Running, Idle, Running)]
#[case(Failure, Idle, Failure)]
#[case(Success, Running, Running)]
#[case(Success, Failure, Failure)]
#[case(Success, Success, Success)]
#[case(Skipped, Skipped, Skipped)]
async fn if_then_else_2_children(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] expected: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input1, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", BehaviorState::Running, input2, 0)?;
	register_behavior!(factory, IfThenElse, "IfThenElse")?;

	let mut tree = factory.create_from_text(TREE_DEFINITION_2_CHILDREN)?;
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
async fn if_then_else_errors(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] input3: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input1, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", BehaviorState::Running, input2, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior3", BehaviorState::Running, input3, 0)?;
	register_behavior!(factory, IfThenElse, "IfThenElse")?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let result = tree.tick_once().await;
	assert!(result.is_err());
	Ok(())
}
