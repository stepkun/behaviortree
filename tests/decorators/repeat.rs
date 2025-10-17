// Copyright © 2025 Stephan Kunz

//! Tests the [`Repeat`] decorator

extern crate alloc;

use crate::decorators::utilities::ChangeStateAfter;
use behaviortree::{
	behavior::{BehaviorState::*, MockBehavior, MockBehaviorConfig},
	prelude::*,
};
use rstest::rstest;

const REPEAT: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Repeat name="root_repeat" num_cycles="{=}">
			<Action	ID="Action" name="action"/>
		</Repeat>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn repeat_raw() -> Result<(), Error> {
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
		return_state: BehaviorState::Success,
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

	let mut tree = factory.create_from_text(REPEAT)?;
	drop(factory);

	tree.blackboard().set("num_cycles", 3)?;
	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	tree.reset()?;
	set_values(&mut tree, BehaviorState::Failure);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);

	Ok(())
}

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
) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	ChangeStateAfter::register(&mut factory, "Behavior1", BehaviorState::Running, input, 0)?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let mut result = tree.tick_once().await?;
	assert_eq!(result, expected);
	result = tree.tick_once().await?;
	assert_eq!(result, finally);
	result = tree.tick_once().await?;
	assert_eq!(result, finally);

	tree.reset()?;

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
async fn repeat_errors(#[case] input: BehaviorState) -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	ChangeStateAfter::register(&mut factory, "Behavior1", BehaviorState::Running, input, 0)?;

	let mut tree = factory.create_from_text(TREE_DEFINITION)?;
	drop(factory);

	let result = tree.tick_once().await;
	assert!(result.is_err());
	Ok(())
}
