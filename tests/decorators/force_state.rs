// Copyright © 2025 Stephan Kunz

//! Tests the [`ForceState`] decorator

extern crate alloc;

use behaviortree::behavior::BehaviorState::*;
use behaviortree::behavior::{action::ChangeStateAfter, decorator::ForceState};
use behaviortree::prelude::*;

use rstest::rstest;

const FORCE_STATE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<ForceState name="force_state">
			<Action	ID="Action" name="action"/>
		</ForceState>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn force_state_raw() -> Result<(), Error> {
	fn set_values(tree: &mut BehaviorTree, force_state: BehaviorState, action_state: BehaviorState) {
		for behavior in tree.iter_mut() {
			if behavior.name().as_ref() == "force_state" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<ForceState>()
				{
					behavior.initialize(force_state);
				}
			}
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
	let bhvr_desc = BehaviorDescription::new(
		"ForceState",
		"ForceState",
		ForceState::kind(),
		true,
		ForceState::provided_ports(),
	);
	let bhvr_creation_fn =
		Box::new(move || -> Box<dyn BehaviorExecution> { Box::new(ForceState::new(BehaviorState::Skipped)) });
	factory
		.registry_mut()
		.add_behavior(bhvr_desc, bhvr_creation_fn)?;

	register_behavior!(
		factory,
		ChangeStateAfter,
		"Action",
		BehaviorState::Running,
		BehaviorState::Failure,
		1
	)?;
	let mut tree = factory.create_from_text(FORCE_STATE)?;
	drop(factory);

	// case 1
	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Skipped);
	// case 2
	set_values(&mut tree, BehaviorState::Success, BehaviorState::Failure);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	// case 2
	set_values(&mut tree, BehaviorState::Success, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	// case 3
	set_values(&mut tree, BehaviorState::Failure, BehaviorState::Success);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	// case 4
	set_values(&mut tree, BehaviorState::Failure, BehaviorState::Failure);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);

	Ok(())
}

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
async fn force_state(#[case] input: BehaviorState, #[case] expected: BehaviorState) -> Result<(), Error> {
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
async fn force_state_errors(#[case] input: BehaviorState) -> Result<(), Error> {
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
