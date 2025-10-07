// Copyright © 2025 Stephan Kunz

//! Tests the [`AsyncSequence`](behaviortree::behavior::control::Sequence) behavior

extern crate alloc;

use behaviortree::{
	behavior::{TestBehavior, TestBehaviorConfig},
	prelude::*,
};

const ASYNC_SEQUENCE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<AsyncSequence name="root_sequence">
			<Action ID="Action" name="action1"/>
			<Action ID="Action" name="action2"/>
			<Action	ID="Action" name="action3"/>
		</AsyncSequence>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn async_sequence_raw() -> Result<(), Error> {
	fn set_values(
		tree: &mut BehaviorTree,
		action1_state: BehaviorState,
		action2_state: BehaviorState,
		action3_state: BehaviorState,
	) {
		for behavior in tree.iter_mut() {
			if behavior.name().as_ref() == "action1" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<TestBehavior>()
				{
					behavior.set_state(action1_state);
				}
			}
			if behavior.name().as_ref() == "action2" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<TestBehavior>()
				{
					behavior.set_state(action2_state);
				}
			}
			if behavior.name().as_ref() == "action3" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<TestBehavior>()
				{
					behavior.set_state(action3_state);
				}
			}
		}
	}

	let mut factory = BehaviorTreeFactory::new()?;

	let config = TestBehaviorConfig {
		return_state: BehaviorState::Failure,
		..Default::default()
	};
	let bhvr_desc = BehaviorDescription::new(
		"Action",
		"Action",
		BehaviorKind::Action,
		false,
		TestBehavior::provided_ports(),
	);
	let bhvr_creation_fn = Box::new(move || -> Box<dyn BehaviorExecution> {
		Box::new(TestBehavior::new(config.clone(), TestBehavior::provided_ports()))
	});
	factory
		.registry_mut()
		.add_behavior(bhvr_desc, bhvr_creation_fn)?;

	let mut tree = factory.create_from_text(ASYNC_SEQUENCE)?;
	drop(factory);

	// case 1
	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	// case 2
	set_values(
		&mut tree,
		BehaviorState::Success,
		BehaviorState::Failure,
		BehaviorState::Success,
	);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	// case 3
	set_values(
		&mut tree,
		BehaviorState::Success,
		BehaviorState::Success,
		BehaviorState::Failure,
	);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	// case 4
	set_values(
		&mut tree,
		BehaviorState::Success,
		BehaviorState::Success,
		BehaviorState::Success,
	);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);

	Ok(())
}
