// Copyright © 2025 Stephan Kunz

//! Tests the [`WhileDoElse`] behavior

extern crate alloc;

use behaviortree::{
    behavior::{
        BehaviorState::{self, *},
        BehaviorStatic,
        action::ChangeStateAfter,
        control::WhileDoElse,
    },
    factory::BehaviorTreeFactory,
    register_behavior,
};

use rstest::rstest;

const TREE_DEFINITION: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<WhileDoElse name="while_do_else">
			<Behavior1	name="if"/>
			<Behavior2	name="then"/>
			<Behavior3	name="else"/>
		</WhileDoElse>
	</BehaviorTree>
</root>
"#;

const TREE_DEFINITION_2_CHILDREN: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<WhileDoElse name="while_do_else">
			<Behavior1	name="if"/>
			<Behavior2	name="then"/>
		</WhileDoElse>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Running, Idle, Idle, Running)]
#[case(Failure, Idle, Running, Running)]
#[case(Failure, Idle, Failure, Failure)]
#[case(Failure, Idle, Success, Success)]
#[case(Success, Running, Idle, Running)]
#[case(Success, Failure, Idle, Failure)]
#[case(Success, Success, Idle, Success)]
#[case(Skipped, Skipped, Skipped, Skipped)]
async fn while_do_else(
    #[case] input1: BehaviorState,
    #[case] input2: BehaviorState,
    #[case] input3: BehaviorState,
    #[case] expected: BehaviorState,
) -> anyhow::Result<()> {
    let mut factory = BehaviorTreeFactory::default();
    register_behavior!(
        factory,
        ChangeStateAfter,
        "Behavior1",
        BehaviorState::Running,
        input1,
        0
    )?;
    register_behavior!(
        factory,
        ChangeStateAfter,
        "Behavior2",
        BehaviorState::Running,
        input2,
        0
    )?;
    register_behavior!(
        factory,
        ChangeStateAfter,
        "Behavior3",
        BehaviorState::Running,
        input3,
        0
    )?;
    register_behavior!(factory, WhileDoElse, "WhileDoElse")?;

    let mut tree = factory.create_from_text(TREE_DEFINITION)?;
    drop(factory);

    let mut result = tree.tick_once().await?;
    assert_eq!(result, expected);
    result = tree.tick_once().await?;
    assert_eq!(result, expected);

    tree.reset().await?;

    result = tree.tick_once().await?;
    assert_eq!(result, expected);
    result = tree.tick_once().await?;
    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
#[rstest]
#[case(Running, Idle, Running)]
#[case(Failure, Idle, Failure)]
#[case(Success, Running, Running)]
#[case(Success, Failure, Failure)]
#[case(Success, Success, Success)]
#[case(Skipped, Skipped, Skipped)]
async fn while_do_else_2_children(
    #[case] input1: BehaviorState,
    #[case] input2: BehaviorState,
    #[case] expected: BehaviorState,
) -> anyhow::Result<()> {
    let mut factory = BehaviorTreeFactory::default();
    register_behavior!(
        factory,
        ChangeStateAfter,
        "Behavior1",
        BehaviorState::Running,
        input1,
        0
    )?;
    register_behavior!(
        factory,
        ChangeStateAfter,
        "Behavior2",
        BehaviorState::Running,
        input2,
        0
    )?;
    register_behavior!(factory, WhileDoElse, "WhileDoElse")?;

    let mut tree = factory.create_from_text(TREE_DEFINITION_2_CHILDREN)?;
    drop(factory);

    let mut result = tree.tick_once().await?;
    assert_eq!(result, expected);
    result = tree.tick_once().await?;
    assert_eq!(result, expected);

    tree.reset().await?;

    result = tree.tick_once().await?;
    assert_eq!(result, expected);
    result = tree.tick_once().await?;
    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
#[rstest]
#[case(Idle, Idle, Idle)]
#[case(Idle, Success, Idle)]
#[case(Idle, Failure, Idle)]
#[case(Idle, Running, Idle)]
#[case(Idle, Skipped, Idle)]
async fn while_do_else_errors(
    #[case] input1: BehaviorState,
    #[case] input2: BehaviorState,
    #[case] input3: BehaviorState,
) -> anyhow::Result<()> {
    let mut factory = BehaviorTreeFactory::default();
    register_behavior!(
        factory,
        ChangeStateAfter,
        "Behavior1",
        BehaviorState::Running,
        input1,
        0
    )?;
    register_behavior!(
        factory,
        ChangeStateAfter,
        "Behavior2",
        BehaviorState::Running,
        input2,
        0
    )?;
    register_behavior!(
        factory,
        ChangeStateAfter,
        "Behavior3",
        BehaviorState::Running,
        input3,
        0
    )?;
    register_behavior!(factory, WhileDoElse, "WhileDoElse")?;

    let mut tree = factory.create_from_text(TREE_DEFINITION)?;
    drop(factory);

    let result = tree.tick_once().await;
    assert!(result.is_err());
    Ok(())
}
