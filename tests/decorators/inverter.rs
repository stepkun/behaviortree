// Copyright © 2025 Stephan Kunz

//! Tests the [`Inverter`] decorator

extern crate alloc;

use behaviortree::prelude::*;
use behaviortree::behavior::BehaviorState::*;
use behaviortree::behavior::{action::ChangeStateAfter, decorator::Inverter};

use rstest::rstest;

const TREE_DEFINITION: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Inverter name="inverter">
			<Behavior1	name="child"/>
		</Inverter>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Running, Running)]
#[case(Skipped, Skipped)]
#[case(Failure, Success)]
#[case(Success, Failure)]
async fn inverter(
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
    register_behavior!(factory, Inverter, "Inverter")?;

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
async fn inverter_errors(#[case] input: BehaviorState) -> Result<(), Error> {
    let mut factory = BehaviorTreeFactory::default();
    register_behavior!(
        factory,
        ChangeStateAfter,
        "Behavior1",
        BehaviorState::Running,
        input,
        0
    )?;
    register_behavior!(factory, Inverter, "Inverter")?;

    let mut tree = factory.create_from_text(TREE_DEFINITION)?;
    drop(factory);

    let result = tree.tick_once().await;
    assert!(result.is_err());
    Ok(())
}
