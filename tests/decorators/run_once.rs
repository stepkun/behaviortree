// Copyright © 2025 Stephan Kunz

//! Tests the [`RunOnce`] decorator

extern crate alloc;

use behaviortree::{
    behavior::{
        BehaviorState::{self, *},
        BehaviorStatic,
        action::ChangeStateAfter,
        decorator::RunOnce,
    },
    factory::BehaviorTreeFactory,
    register_behavior,
};

use rstest::rstest;

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
async fn run_once(#[case] input: BehaviorState) -> anyhow::Result<()> {
    let mut factory = BehaviorTreeFactory::default();
    register_behavior!(
        factory,
        ChangeStateAfter,
        "Behavior1",
        BehaviorState::Running,
        input,
        0
    )?;
    register_behavior!(factory, RunOnce, "RunOnce")?;

    let mut tree = factory.create_from_text(TREE_DEFINITION)?;
    drop(factory);

    let mut result = tree.tick_once().await?;
    assert_eq!(result, input);
    result = tree.tick_once().await?;
    assert_eq!(result, BehaviorState::Skipped);

    tree.reset().await?;

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
async fn run_once_no_skip(#[case] input: BehaviorState) -> anyhow::Result<()> {
    let mut factory = BehaviorTreeFactory::default();
    register_behavior!(
        factory,
        ChangeStateAfter,
        "Behavior1",
        BehaviorState::Running,
        input,
        0
    )?;
    register_behavior!(factory, RunOnce, "RunOnce")?;

    let mut tree = factory.create_from_text(TREE_DEFINITION2)?;
    drop(factory);

    let mut result = tree.tick_once().await?;
    assert_eq!(result, input);
    result = tree.tick_once().await?;
    assert_eq!(result, input);

    tree.reset().await?;

    result = tree.tick_once().await?;
    assert_eq!(result, input);
    result = tree.tick_once().await?;
    assert_eq!(result, input);

    Ok(())
}

#[tokio::test]
#[rstest]
#[case(Idle)]
async fn run_once_errors(#[case] input: BehaviorState) -> anyhow::Result<()> {
    let mut factory = BehaviorTreeFactory::default();
    register_behavior!(
        factory,
        ChangeStateAfter,
        "Behavior1",
        BehaviorState::Running,
        input,
        0
    )?;
    register_behavior!(factory, RunOnce, "RunOnce")?;

    let mut tree = factory.create_from_text(TREE_DEFINITION)?;
    drop(factory);

    let result = tree.tick_once().await;
    assert!(result.is_err());
    Ok(())
}
