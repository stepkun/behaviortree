// Copyright © 2025 Stephan Kunz

//! Tests the [`Sequence`](behaviortree::behavior::control::Sequence) behavior

extern crate alloc;

use crate::controls::utilities::ChangeStateAfter;
use behaviortree::{
	behavior::{BehaviorState::*, MockBehavior, MockBehaviorConfig},
	prelude::*,
};
use rstest::rstest;

const SEQUENCE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Sequence name="root_sequence">
			<Action ID="Action" name="action1"/>
			<Action ID="Action" name="action2"/>
			<Action	ID="Action" name="action3"/>
		</Sequence>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn simple_sequence_raw() -> Result<(), Error> {
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
					.downcast_mut::<MockBehavior>()
				{
					behavior.set_state(action1_state);
				}
			}
			if behavior.name().as_ref() == "action2" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<MockBehavior>()
				{
					behavior.set_state(action2_state);
				}
			}
			if behavior.name().as_ref() == "action3" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<MockBehavior>()
				{
					behavior.set_state(action3_state);
				}
			}
		}
	}

	let mut factory = BehaviorTreeFactory::new()?;

	let config = MockBehaviorConfig {
		return_state: BehaviorState::Failure,
		..Default::default()
	};
	let bhvr_desc = BehaviorDescription::new(
		"Action",
		"Action",
		BehaviorKind::Action,
		false,
		MockBehavior::provided_ports(),
	);
	let bhvr_creation_fn = Box::new(move || -> Box<dyn BehaviorExecution> {
		Box::new(MockBehavior::new(config.clone(), MockBehavior::provided_ports()))
	});
	factory
		.registry_mut()
		.add_behavior(bhvr_desc, bhvr_creation_fn)?;

	let mut tree = factory.create_from_text(SEQUENCE)?;
	drop(factory);

	// case 1
	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	// case 2
	set_values(
		&mut tree,
		BehaviorState::Success,
		BehaviorState::Failure,
		BehaviorState::Success,
	);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	// case 3
	set_values(
		&mut tree,
		BehaviorState::Success,
		BehaviorState::Success,
		BehaviorState::Success,
	);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	set_values(
		&mut tree,
		BehaviorState::Failure,
		BehaviorState::Success,
		BehaviorState::Success,
	);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);

	Ok(())
}

const TREE_DEFINITION: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Sequence name="simple_sequence">
			<Behavior1	name="step1"/>
			<Behavior2	name="step2"/>
			<Behavior3	name="step3"/>
		</Sequence>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Running, Idle, Idle, Running)]
#[case(Failure, Running, Idle, Failure)]
#[case(Failure, Failure, Running, Failure)]
#[case(Failure, Failure, Failure, Failure)]
#[case(Success, Running, Idle, Running)]
#[case(Success, Success, Running, Running)]
#[case(Success, Failure, Success, Failure)]
#[case(Success, Success, Failure, Failure)]
#[case(Failure, Success, Idle, Failure)]
#[case(Success, Running, Failure, Running)]
#[case(Skipped, Skipped, Success, Success)]
#[case(Skipped, Skipped, Skipped, Skipped)]
#[case(Skipped, Skipped, Running, Running)]
#[case(Skipped, Skipped, Failure, Failure)]
#[case(Success, Skipped, Success, Success)]
#[case(Success, Success, Success, Success)]
async fn simple_sequence(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] input3: BehaviorState,
	#[case] expected: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	ChangeStateAfter::register(&mut factory, "Behavior1", BehaviorState::Running, input1, 0)?;
	ChangeStateAfter::register(&mut factory, "Behavior2", BehaviorState::Running, input2, 0)?;
	ChangeStateAfter::register(&mut factory, "Behavior3", BehaviorState::Running, input3, 0)?;

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
async fn simple_sequence_errors(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] input3: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	ChangeStateAfter::register(&mut factory, "Behavior1", BehaviorState::Running, input1, 0)?;
	ChangeStateAfter::register(&mut factory, "Behavior2", BehaviorState::Running, input2, 0)?;
	ChangeStateAfter::register(&mut factory, "Behavior3", BehaviorState::Running, input3, 0)?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let result = tree.tick_once().await;
	assert!(result.is_err());
	Ok(())
}

#[tokio::test]
#[rstest]
#[case(Success, Failure, Running, Running, Running, Success)]
#[case(Failure, Success, Failure, Failure, Failure, Failure)]
async fn simple_sequence_reactiveness1(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] expected1: BehaviorState,
	#[case] expected2: BehaviorState,
	#[case] expected3: BehaviorState,
	#[case] expected4: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	ChangeStateAfter::register(&mut factory, "Behavior1", input1, input2, 1)?;
	ChangeStateAfter::register(&mut factory, "Behavior2", input1, input2, 2)?;
	ChangeStateAfter::register(&mut factory, "Behavior3", input1, input2, 3)?;

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
#[case(Success, Failure, Running, Running, Running, Success)]
#[case(Failure, Success, Running, Running, Failure, Running)]
async fn simple_sequence_reactiveness2(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] expected1: BehaviorState,
	#[case] expected2: BehaviorState,
	#[case] expected3: BehaviorState,
	#[case] expected4: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	ChangeStateAfter::register(&mut factory, "Behavior1", input1, input2, 3)?;
	ChangeStateAfter::register(&mut factory, "Behavior2", input1, input2, 2)?;
	ChangeStateAfter::register(&mut factory, "Behavior3", input1, input2, 1)?;

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
