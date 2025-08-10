// Copyright © 2025 Stephan Kunz

//! Tests the [`KeepRunningUntilFailure`] decorator

extern crate alloc;

use behaviortree::prelude::*;
use behaviortree::behavior::BehaviorState::*;
use behaviortree::behavior::{action::ChangeStateAfter, decorator::KeepRunningUntilFailure};

use rstest::rstest;

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
async fn keep_runnning_until_failure(
    #[case] input: BehaviorState,
    #[case] expected: BehaviorState,
) -> Result<(), Error> {
    let mut factory = BehaviorTreeFactory::default();
    register_behavior!(
        factory,
        ChangeStateAfter,
        "Behavior1",
        BehaviorState::Running,
        input,
        0
    )?;
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
    let mut factory = BehaviorTreeFactory::default();
    register_behavior!(
        factory,
        ChangeStateAfter,
        "Behavior1",
        BehaviorState::Running,
        input,
        0
    )?;
    register_behavior!(factory, KeepRunningUntilFailure, "KeepRunningUntilFailure")?;

    let mut tree = factory.create_from_text(TREE_DEFINITION)?;
    drop(factory);

    let result = tree.tick_once().await;
    assert!(result.is_err());
    Ok(())
}
