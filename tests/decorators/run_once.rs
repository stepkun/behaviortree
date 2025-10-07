// Copyright © 2025 Stephan Kunz

//! Tests the [`RunOnce`] decorator

extern crate alloc;

use behaviortree::{
	behavior::{BehaviorState::*, ChangeStateAfter},
	prelude::*,
};
use rstest::rstest;

const RUN_ONCE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<RunOnce name="root_run_once" then_skip="{=}">
			<Action	ID="Action" name="action"/>
		</RunOnce>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn run_once_raw() -> Result<(), Error> {
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

	let mut tree = factory.create_from_text(RUN_ONCE)?;
	drop(factory);

	// case 1
	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Skipped);
	// case 2
	tree.reset()?;
	set_values(&mut tree, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Skipped);
	// case 3
	tree.blackboard().set("then_skip", false)?;
	tree.reset()?;
	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	// case 4
	tree.reset()?;
	set_values(&mut tree, BehaviorState::Failure);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);

	Ok(())
}

const TREE_DEFINITION: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<RunOnce name="run_once">
			<Behavior1	name="child"/>
		</RunOnce>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Skipped)]
#[case(Failure)]
#[case(Success)]
async fn run_once(#[case] input: BehaviorState) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input, 0)?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let mut result = tree.tick_once().await?;
	assert_eq!(result, input);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Skipped);

	tree.reset()?;

	result = tree.tick_once().await?;
	assert_eq!(result, input);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Skipped);

	Ok(())
}

const TREE_DEFINITION2: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<RunOnce name="run_once_no_skip" then_skip="false">
			<Behavior1	name="child"/>
		</RunOnce>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Skipped)]
#[case(Failure)]
#[case(Success)]
async fn run_once_no_skip(#[case] input: BehaviorState) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input, 0)?;

	let mut tree = factory.create_from_text(TREE_DEFINITION2)?;
	drop(factory);

	let mut result = tree.tick_once().await?;
	assert_eq!(result, input);
	result = tree.tick_once().await?;
	assert_eq!(result, input);

	tree.reset()?;

	result = tree.tick_once().await?;
	assert_eq!(result, input);
	result = tree.tick_once().await?;
	assert_eq!(result, input);

	Ok(())
}

#[tokio::test]
#[rstest]
#[case(Idle)]
async fn run_once_errors(#[case] input: BehaviorState) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input, 0)?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let result = tree.tick_once().await;
	assert!(result.is_err());
	Ok(())
}
