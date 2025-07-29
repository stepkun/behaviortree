// Copyright © 2025 Stephan Kunz

//! Tests the [`Repeat`] decorator

extern crate alloc;

use behaviortree::{
    behavior::{
        BehaviorState::{self, *},
        BehaviorStatic,
        action::ChangeStateAfter,
        decorator::Repeat,
    },
    factory::BehaviorTreeFactory,
    register_behavior,
};

use rstest::rstest;

const TREE_DEFINITION: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Repeat name="repeat" num_cycles="2">
			<Behavior1	name="child"/>
		</Repeat>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Running, Running, Running)]
#[case(Skipped, Skipped, Skipped)]
#[case(Failure, Failure, Failure)]
#[case(Success, Running, Success)]
async fn repeat(
    #[case] input: BehaviorState,
    #[case] expected: BehaviorState,
    #[case] finally: BehaviorState,
) -> anyhow::Result<()> {
    let mut factory = BehaviorTreeFactory::default();
    register_behavior!(
        factory,
        ChangeStateAfter,
        "Behavior1",
        BehaviorState::Running,
        input,
        0
    )?;
    register_behavior!(factory, Repeat, "Repeat")?;

    let mut tree = factory.create_from_text(TREE_DEFINITION)?;
    drop(factory);

    let mut result = tree.tick_once().await?;
    assert_eq!(result, expected);
    result = tree.tick_once().await?;
    assert_eq!(result, expected);
    result = tree.tick_once().await?;
    assert_eq!(result, finally);
    result = tree.tick_once().await?;
    assert_eq!(result, finally);

    tree.reset()?;

    result = tree.tick_once().await?;
    assert_eq!(result, expected);
    result = tree.tick_once().await?;
    assert_eq!(result, expected);
    result = tree.tick_once().await?;
    assert_eq!(result, finally);
    result = tree.tick_once().await?;
    assert_eq!(result, finally);

    Ok(())
}

#[tokio::test]
#[rstest]
#[case(Idle)]
async fn repeat_errors(#[case] input: BehaviorState) -> anyhow::Result<()> {
    let mut factory = BehaviorTreeFactory::default();
    register_behavior!(
        factory,
        ChangeStateAfter,
        "Behavior1",
        BehaviorState::Running,
        input,
        0
    )?;
    register_behavior!(factory, Repeat, "Repeat")?;

    let mut tree = factory.create_from_text(TREE_DEFINITION)?;
    drop(factory);

    let result = tree.tick_once().await;
    assert!(result.is_err());
    Ok(())
}
