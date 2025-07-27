// Copyright © 2025 Stephan Kunz

//! Tests the [`Switch`] behavior

extern crate alloc;

use behaviortree::{
	behavior::{
		BehaviorState::{self, *},
		BehaviorStatic,
		action::ChangeStateAfter,
		control::Switch,
	},
	blackboard::BlackboardInterface,
	factory::BehaviorTreeFactory,
	register_behavior,
};

use rstest::rstest;

const TREE_DEFINITION_4_CHILDREN: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Switch4 name="switch4">
			<Behavior1	name="case1"/>
			<Behavior2	name="case2"/>
			<Behavior3	name="case3"/>
			<Behavior4	name="case4"/>
			<Default	name="default"/>
		</Switch4>
	</BehaviorTree>
</root>
"#;

const TREE_DEFINITION_2_CHILDREN: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Switch2 name="switch2">
			<Behavior1	name="case1"/>
			<Behavior2	name="case2"/>
			<Default	name="default"/>
		</Switch2>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Idle, Idle, Idle, Idle, Running, "default", Running)]
#[case(Idle, Idle, Idle, Idle, Failure, "default", Failure)]
#[case(Idle, Idle, Idle, Idle, Skipped, "default", Skipped)]
#[case(Idle, Idle, Idle, Idle, Success, "default", Success)]
#[case(Success, Failure, Failure, Failure, Failure, "case_0", Success)]
#[case(Failure, Success, Failure, Failure, Failure, "case_1", Success)]
#[case(Failure, Failure, Success, Failure, Failure, "case_2", Success)]
#[case(Failure, Failure, Failure, Success, Failure, "case_3", Success)]
async fn switch4(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] input3: BehaviorState,
	#[case] input4: BehaviorState,
	#[case] default: BehaviorState,
	#[case] case: &str,
	#[case] expected: BehaviorState,
) -> anyhow::Result<()> {
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input1, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", BehaviorState::Running, input2, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior3", BehaviorState::Running, input3, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior4", BehaviorState::Running, input4, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Default", BehaviorState::Running, default, 0)?;
	factory.register_behavior_type::<Switch<4>>("Switch4")?;

	let mut tree = factory.create_from_text(TREE_DEFINITION_4_CHILDREN)?;
	drop(factory);

	tree.blackboard_mut()
		.set("case_0", String::from("case_0"))?;
	tree.blackboard_mut()
		.set("case_1", String::from("case_1"))?;
	tree.blackboard_mut()
		.set("case_2", String::from("case_2"))?;
	tree.blackboard_mut()
		.set("case_3", String::from("case_3"))?;
	tree.blackboard_mut()
		.set("variable", String::from(case))?;

	let mut result = tree.tick_once().await?;
	assert_eq!(result, expected);
	result = tree.tick_once().await?;
	assert_eq!(result, expected);

	tree.reset().await?;

	result = tree.tick_once().await?;
	assert_eq!(result, expected);
	result = tree.tick_once().await?;
	assert_eq!(result, expected);

	Ok(())
}

#[tokio::test]
#[rstest]
#[case(Idle, Idle, Running, "default", Running)]
#[case(Idle, Idle, Failure, "default", Failure)]
#[case(Idle, Idle, Skipped, "default", Skipped)]
#[case(Idle, Idle, Success, "default", Success)]
#[case(Success, Failure, Failure, "case_0", Success)]
#[case(Failure, Success, Failure, "case_1", Success)]
async fn switch2(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] default: BehaviorState,
	#[case] case: &str,
	#[case] expected: BehaviorState,
) -> anyhow::Result<()> {
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input1, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", BehaviorState::Running, input2, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Default", BehaviorState::Running, default, 0)?;
	factory.register_behavior_type::<Switch<2>>("Switch2")?;

	let mut tree = factory.create_from_text(TREE_DEFINITION_2_CHILDREN)?;
	drop(factory);

	tree.blackboard_mut()
		.set("case_0", String::from("case_0"))?;
	tree.blackboard_mut()
		.set("case_1", String::from("case_1"))?;
	tree.blackboard_mut()
		.set("variable", String::from(case))?;

	let mut result = tree.tick_once().await?;
	assert_eq!(result, expected);
	result = tree.tick_once().await?;
	assert_eq!(result, expected);

	tree.reset().await?;

	result = tree.tick_once().await?;
	assert_eq!(result, expected);
	result = tree.tick_once().await?;
	assert_eq!(result, expected);

	Ok(())
}

#[tokio::test]
#[rstest]
#[case(Idle, Idle, Idle)]
async fn switch_errors(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] default: BehaviorState,
) -> anyhow::Result<()> {
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input1, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", BehaviorState::Running, input2, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Default", BehaviorState::Running, default, 0)?;
	factory.register_behavior_type::<Switch<2>>("Switch2")?;

	let mut tree = factory.create_from_text(TREE_DEFINITION_2_CHILDREN)?;
	drop(factory);

	let result = tree.tick_once().await;
	assert!(result.is_err());
	Ok(())
}
