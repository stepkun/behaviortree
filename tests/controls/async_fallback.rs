// Copyright © 2025 Stephan Kunz

//! Tests the [`AsyncFallback`](behaviortree::behavior::control::Fallback) behavior

extern crate alloc;

use behaviortree::{
	behavior::{TestBehavior, TestBehaviorConfig},
	prelude::*,
};

const ASYNC_FALLBACK: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<AsyncFallback name="root_fallback">
			<Condition ID="Condition" name="condition1"/>
			<Condition ID="Condition" name="condition2"/>
			<Action	ID="Action" name="action"/>
		</AsyncFallback>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn async_fallback_raw() -> Result<(), Error> {
	fn set_values(
		tree: &mut BehaviorTree,
		condition1_state: BehaviorState,
		condition2_state: BehaviorState,
		action_state: BehaviorState,
	) {
		for behavior in tree.iter_mut() {
			if behavior.name().as_ref() == "condition1" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<TestBehavior>()
				{
					behavior.set_state(condition1_state);
				}
			}
			if behavior.name().as_ref() == "condition2" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<TestBehavior>()
				{
					behavior.set_state(condition2_state);
				}
			}
			if behavior.name().as_ref() == "action" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<TestBehavior>()
				{
					behavior.set_state(action_state);
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
		"Condition",
		"Condition",
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

	let mut tree = factory.create_from_text(ASYNC_FALLBACK)?;
	drop(factory);

	// case 1
	let mut result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Failure);
	// case 2
	set_values(
		&mut tree,
		BehaviorState::Success,
		BehaviorState::Failure,
		BehaviorState::Failure,
	);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	// case 3
	set_values(
		&mut tree,
		BehaviorState::Failure,
		BehaviorState::Success,
		BehaviorState::Failure,
	);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Running);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);
	// case 4
	set_values(
		&mut tree,
		BehaviorState::Success,
		BehaviorState::Success,
		BehaviorState::Failure,
	);
	result = tree.tick_once().await?;
	assert_eq!(result, BehaviorState::Success);

	Ok(())
}
