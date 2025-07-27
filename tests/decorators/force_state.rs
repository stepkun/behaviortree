// Copyright © 2025 Stephan Kunz

//! Tests the [`ForceState`] decorator

extern crate alloc;

use behaviortree::{
	behavior::{
		BehaviorDescription, BehaviorExecution,
		BehaviorState::{self, *},
		BehaviorStatic,
		action::ChangeStateAfter,
		decorator::ForceState,
	},
	factory::BehaviorTreeFactory,
	register_behavior,
};

use rstest::rstest;

const TREE_DEFINITION: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<ForceState name="force_state">
			<Behavior1	name="child"/>
		</ForceState>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[rstest]
#[case(Running, Running)]
#[case(Skipped, Skipped)]
#[case(Failure, Failure)]
#[case(Success, Failure)]
#[case(Failure, Success)]
#[case(Success, Success)]
async fn force_state(#[case] input: BehaviorState, #[case] expected: BehaviorState) -> anyhow::Result<()> {
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input, 0)?;
	let bhvr_desc = BehaviorDescription::new(
		"ForceState",
		"ForceState",
		ForceState::kind(),
		true,
		ForceState::provided_ports(),
	);
	let bhvr_creation_fn = Box::new(move || -> Box<dyn BehaviorExecution> { Box::new(ForceState::new(expected)) });
	factory
		.registry_mut()
		.add_behavior(bhvr_desc, bhvr_creation_fn)?;

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
#[case(Idle)]
async fn force_state_errors(#[case] input: BehaviorState) -> anyhow::Result<()> {
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, ChangeStateAfter, "Behavior1", BehaviorState::Running, input, 0)?;
	let bhvr_desc = BehaviorDescription::new(
		"ForceState",
		"ForceState",
		ForceState::kind(),
		true,
		ForceState::provided_ports(),
	);
	let bhvr_creation_fn = Box::new(move || -> Box<dyn BehaviorExecution> { Box::new(ForceState::new(input)) });
	factory
		.registry_mut()
		.add_behavior(bhvr_desc, bhvr_creation_fn)?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let result = tree.tick_once().await;
	assert!(result.is_err());
	Ok(())
}
