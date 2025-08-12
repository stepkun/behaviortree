// Copyright © 2025 Stephan Kunz

//! Tests the [`RetryUntilSuccessful`] decorator

extern crate alloc;

use behaviortree::behavior::BehaviorState::*;
use behaviortree::behavior::{action::ChangeStateAfter, decorator::RetryUntilSuccessful};
use behaviortree::prelude::*;

use rstest::rstest;

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
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Failure, input, 0)?;
	register_behavior!(factory, RetryUntilSuccessful, "RetryUntilSuccessful")?;

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
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input, 0)?;
	register_behavior!(factory, RetryUntilSuccessful, "RetryUntilSuccessful")?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let result = tree.tick_once().await;
	assert!(result.is_err());
	Ok(())
}
