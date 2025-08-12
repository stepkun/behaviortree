// Copyright © 2025 Stephan Kunz
#![allow(clippy::too_many_arguments)]

//! Tests the [`Switch`] behavior

extern crate alloc;

use behaviortree::behavior::BehaviorState::*;
use behaviortree::behavior::{action::ChangeStateAfter, control::Switch};
use behaviortree::prelude::*;

use rstest::rstest;

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
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, ChangeStateAfter, "Behavior1", Running, input1, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", Running, input2, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Default", Running, default, 0)?;
	factory.register_behavior_type::<Switch<2>>("Switch2")?;

	let mut tree = factory.create_from_text(SWITCH2_TREE)?;
	drop(factory);

	tree.blackboard_mut()
		.set("var", String::from(case))?;

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
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, ChangeStateAfter, "Behavior1", Running, Idle, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", Running, Idle, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Default", Running, Idle, 0)?;
	factory.register_behavior_type::<Switch<2>>("Switch2")?;

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
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, ChangeStateAfter, "Behavior1", Running, input1, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", Running, input2, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior3", Running, input3, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior4", Running, input4, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior5", Running, input5, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Default", Running, default, 0)?;
	factory.register_behavior_type::<Switch<5>>("Switch5")?;

	// register the Enum values: BLUE=2
	factory.register_enum_tuple("GREEN", 1)?;
	factory.register_enum_tuple("BLUE", 2)?;

	let mut tree = factory.create_from_text(SWITCH_5_TREE)?;
	drop(factory);

	tree.blackboard_mut()
		.set("var", String::from(case))?;

	// set the bb value 'the_answer'
	tree.blackboard_mut().set("the_answer", 42)?;

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
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, ChangeStateAfter, "Behavior1", Running, Success, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Behavior2", Running, Success, 0)?;
	register_behavior!(factory, ChangeStateAfter, "Default", Running, Success, 0)?;
	factory.register_behavior_type::<Switch<2>>("Switch2")?;

	let mut tree = factory.create_from_text(WRONG_TREE)?;
	drop(factory);

	let result = tree.tick_once().await;
	assert!(result.is_err());
	Ok(())
}
