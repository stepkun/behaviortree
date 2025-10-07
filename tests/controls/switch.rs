// Copyright © 2025 Stephan Kunz
#![allow(clippy::too_many_arguments)]

//! Tests the [`Switch`] behavior

extern crate alloc;

use behaviortree::{
	behavior::{BehaviorState::*, ChangeStateAfter},
	prelude::*,
};
use rstest::rstest;

const SWITCH: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Switch3 name="root_switch" variable="{var}"  case_1="1" case_2="2" case_3="3">
			<Action ID="Action" name="case1"/>
			<Action ID="Action" name="case2"/>
			<Action	ID="Action" name="case3"/>
			<Action	ID="Action" name="default"/>
		</Switch3>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn switch_raw() -> Result<(), Error> {
	fn set_values(
		tree: &mut BehaviorTree,
		action1_state: BehaviorState,
		action2_state: BehaviorState,
		action3_state: BehaviorState,
		default_state: BehaviorState,
	) {
		for behavior in tree.iter_mut() {
			if behavior.name().as_ref() == "case1" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<ChangeStateAfter>()
				{
					behavior.set_final_state(action1_state);
				}
			}
			if behavior.name().as_ref() == "case2" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<ChangeStateAfter>()
				{
					behavior.set_final_state(action2_state);
				}
			}
			if behavior.name().as_ref() == "case3" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<ChangeStateAfter>()
				{
					behavior.set_final_state(action3_state);
				}
			}
			if behavior.name().as_ref() == "default" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<ChangeStateAfter>()
				{
					behavior.set_final_state(default_state);
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
	factory.register_behavior_tree_from_text(SWITCH)?;

	let blackboard = Databoard::new();
	let mut tree = factory.create_tree_with("MainTree", &blackboard)?;
	drop(factory);

	// preparation
	blackboard.set("var", String::from("1"))?;
	set_values(
		&mut tree,
		BehaviorState::Success,
		BehaviorState::Failure,
		BehaviorState::Running,
		BehaviorState::Skipped,
	);
	// case 1
	// blackboard.set("var", 1)?;
	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	// case 2
	blackboard.set("var", String::from("2"))?;
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	// case 3
	blackboard.set("var", String::from("3"))?;
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	// default
	blackboard.set("var", String::from("42"))?;
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Skipped);

	Ok(())
}

const SWITCH2_TREE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Switch2 name="switch2" variable="{var}"  case_1="1" case_2="42">
			<Behavior1	name="case1"/>
			<Behavior2	name="case2"/>
			<Default	name="default"/>
		</Switch2>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Idle, Idle, Running, "default", Running)]
#[case(Idle, Idle, Success, "default", Success)]
#[case(Success, Failure, Failure, "1", Success)]
#[case(Failure, Success, Failure, "42", Success)]
async fn switch2(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] default: BehaviorState,
	#[case] case: &str,
	#[case] expected: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", Running, input1, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", Running, input2, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Default", Running, default, 0)?;

	let mut tree = factory.create_from_text(SWITCH2_TREE)?;
	drop(factory);

	tree.blackboard().set("var", String::from(case))?;

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
async fn switch_state_errors() -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", Running, Idle, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", Running, Idle, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Default", Running, Idle, 0)?;

	let mut tree = factory.create_from_text(SWITCH2_TREE)?;
	drop(factory);

	let result = tree.tick_once().await;
	assert!(result.is_err());
	Ok(())
}

const SWITCH_5_TREE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Switch5 name="switch5" variable="{var}" case_1="string" case_2="1" case_3="BLUE" case_4="3.14" case_5="{the_answer}" >
			<Behavior1	name="case1"/>
			<Behavior2	name="case2"/>
			<Behavior3	name="case3"/>
			<Behavior4	name="case4"/>
			<Behavior5	name="case5"/>
			<Default	name="default"/>
		</Switch5>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Idle, Idle, Idle, Idle, Idle, Running, "default", Running)]
#[case(Idle, Idle, Idle, Idle, Idle, Success, "default", Success)]
#[case(Success, Success, Success, Success, Success, Failure, "24", Failure)]
#[case(Success, Failure, Failure, Failure, Failure, Failure, "string", Success)]
#[case(Failure, Success, Failure, Failure, Failure, Failure, "1", Success)]
#[case(Failure, Failure, Success, Failure, Failure, Failure, "BLUE", Success)]
#[case(Failure, Failure, Success, Failure, Failure, Failure, "2", Success)]
#[case(Idle, Idle, Idle, Idle, Idle, Failure, "3", Failure)]
#[case(Failure, Failure, Failure, Success, Failure, Failure, "3.1399999999999996", Success)] // 09
#[case(Failure, Failure, Failure, Failure, Success, Failure, "42", Success)]
#[case(Failure, Failure, Failure, Failure, Success, Failure, "the_answer", Failure)]
#[case(Failure, Failure, Failure, Failure, Success, Failure, "{the_answer}", Failure)]
async fn switch5(
	#[case] input1: BehaviorState,
	#[case] input2: BehaviorState,
	#[case] input3: BehaviorState,
	#[case] input4: BehaviorState,
	#[case] input5: BehaviorState,
	#[case] default: BehaviorState,
	#[case] case: &str,
	#[case] expected: BehaviorState,
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", Running, input1, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", Running, input2, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior3", Running, input3, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior4", Running, input4, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior5", Running, input5, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Default", Running, default, 0)?;

	// register the Enum values: BLUE=2
	factory.register_enum_tuple("GREEN", 1)?;
	factory.register_enum_tuple("BLUE", 2)?;

	let mut tree = factory.create_from_text(SWITCH_5_TREE)?;
	drop(factory);

	tree.blackboard().set("var", String::from(case))?;

	// set the bb value 'the_answer'
	tree.blackboard().set("the_answer", 42)?;

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

const WRONG_TREE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Switch2 name="switch2" variable="var"  case_1="1" case_2="42">
			<Behavior1	name="case1"/>
			<Behavior2	name="case2"/>
			<Default	name="default"/>
		</Switch2>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn switch_wrong_variable_definition() -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", Running, Success, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", Running, Success, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Default", Running, Success, 0)?;
	let mut tree = factory.create_from_text(WRONG_TREE)?;
	drop(factory);

	let result = tree.tick_once().await;
	assert!(result.is_err());
	Ok(())
}
