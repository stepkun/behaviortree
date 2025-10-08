// Copyright © 2025 Stephan Kunz

//! Tests the [`RetryUntilSuccessful`] decorator

extern crate alloc;

use crate::decorators::utilities::ChangeStateAfter;
use behaviortree::{
	behavior::{BehaviorState::*, MockBehavior, MockBehaviorConfig},
	prelude::*,
};
use rstest::rstest;

const RETRY_UNTIL_SUCCESSFUL: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<RetryUntilSuccessful name="root_retry_until_successful" num_attempts="{num_attempts}">
			<Action	ID="Action" name="action"/>
		</RetryUntilSuccessful>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn retry_until_successful_raw() -> Result<(), Error> {
	fn set_values(tree: &mut BehaviorTree, action_state: BehaviorState) {
		for behavior in tree.iter_mut() {
			if behavior.name().as_ref() == "action" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<MockBehavior>()
				{
					behavior.set_state(action_state);
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

	let mut tree = factory.create_from_text(RETRY_UNTIL_SUCCESSFUL)?;
	drop(factory);

	tree.blackboard().set("num_attempts", 3)?;
	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);

	tree.reset()?;
	tree.blackboard().set("num_attempts", 2)?;
	set_values(&mut tree, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);

	Ok(())
}

const TREE_DEFINITION: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<RetryUntilSuccessful name="retry_until_successsful" num_attempts="2">
			<Behavior1	name="child"/>
		</RetryUntilSuccessful>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Running, Running)]
#[case(Skipped, Skipped)]
#[case(Failure, Failure)]
#[case(Success, Success)]
async fn retry_until_successful(#[case] input: BehaviorState, #[case] expected: BehaviorState) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Failure, input, 0)?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	tree.tick_once().await?;
	let mut result = tree.tick_once().await?;
	assert_eq!(result, expected);
	result = tree.tick_once().await?;
	assert_eq!(result, expected);

	tree.reset()?;

	tree.tick_once().await?;
	result = tree.tick_once().await?;
	assert_eq!(result, expected);
	result = tree.tick_once().await?;
	assert_eq!(result, expected);

	Ok(())
}

#[tokio::test]
#[rstest]
#[case(Idle)]
async fn retry_until_successful_errors(#[case] input: BehaviorState) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input, 0)?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let result = tree.tick_once().await;
	assert!(result.is_err());
	Ok(())
}
