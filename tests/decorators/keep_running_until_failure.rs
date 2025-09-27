// Copyright © 2025 Stephan Kunz

//! Tests the [`KeepRunningUntilFailure`] decorator

extern crate alloc;

use behaviortree::behavior::BehaviorState::*;
use behaviortree::behavior::{action::ChangeStateAfter, decorator::KeepRunningUntilFailure};
use behaviortree::prelude::*;

use rstest::rstest;

const KEEP_RUNNING_UNTIL_FAILURE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<KeepRunningUntilFailure name="keep_running_until_failure">
			<Action	ID="Action" name="action"/>
		</KeepRunningUntilFailure>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn keep_running_until_failure_raw() -> Result<(), Error> {
	fn set_values(tree: &mut BehaviorTree, action_state: BehaviorState) {
		for behavior in tree.iter_mut() {
			if behavior.name().as_ref() == "action" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<ChangeStateAfter>()
				{
					behavior.set_state1(action_state);
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
		BehaviorState::Success,
		BehaviorState::Success,
		3
	)?;
	register_behavior!(factory, KeepRunningUntilFailure, "KeepRunningUntilFailure")?;

	let mut tree = factory.create_from_text(KEEP_RUNNING_UNTIL_FAILURE)?;
	drop(factory);

	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	tree.reset()?;
	set_values(&mut tree, BehaviorState::Failure);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);

	Ok(())
}

const TREE_DEFINITION: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<KeepRunningUntilFailure name="keep_running_until_failure">
			<Behavior1	name="child"/>
		</KeepRunningUntilFailure>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Running, Running)]
#[case(Failure, Failure)]
#[case(Success, Running)]
async fn keep_runnning_until_failure(#[case] input: BehaviorState, #[case] expected: BehaviorState) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input, 0)?;
	register_behavior!(factory, KeepRunningUntilFailure, "KeepRunningUntilFailure")?;

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
#[case(Skipped)]
async fn keep_runnning_until_failure_errors(#[case] input: BehaviorState) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input, 0)?;
	register_behavior!(factory, KeepRunningUntilFailure, "KeepRunningUntilFailure")?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let result = tree.tick_once().await;
	assert!(result.is_err());
	Ok(())
}
