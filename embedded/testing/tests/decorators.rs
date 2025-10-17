// Copyright © 2025 Stephan Kunz
//! Embedded control tests.

#![no_main]
#![no_std]
#![allow(clippy::unwrap_used)]

extern crate alloc;

#[cfg(test)]
#[embedded_test::tests]
mod tests {
	use behaviortree::{
		behavior::{
			MockBehavior, MockBehaviorConfig, SharedQueue,
			decorator::{EntryUpdated, ForceState},
		},
		prelude::*,
	};

	const ENTRY_UPDATED: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<EntryUpdated name="entry_updated" entry="test">
			<Action	ID="Action" name="action"/>
		</EntryUpdated>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn entry_updated() -> Result<(), Error> {
		let mut factory = BehaviorTreeFactory::new()?;

		EntryUpdated::register(&mut factory, "EntryUpdated", BehaviorState::Idle, true)?;
		MockBehavior::register(&mut factory, "Action", MockBehaviorConfig::new(BehaviorState::Success), true)?;

		let mut tree = factory.create_from_text(ENTRY_UPDATED)?;
		drop(factory);

		tree.blackboard().set("test", 1)?;
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Idle);
		tree.blackboard().set("test", 2)?;
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Idle);
		for behavior in tree.iter_mut() {
			if behavior.name().as_ref() == "entry_updated" {
				if let Some(behavior) = behavior
					.behavior_mut()
					.as_any_mut()
					.downcast_mut::<EntryUpdated>()
				{
					behavior.initialize(BehaviorState::Failure);
				}
			}
		}
		tree.blackboard().set("test", 1)?;
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		tree.blackboard().set("test", 2)?;
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);

		Ok(())
	}

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

	#[test]
	async fn force_state() -> Result<(), Error> {
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
						.downcast_mut::<MockBehavior>()
					{
						behavior.set_state(action_state);
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

		let config = MockBehaviorConfig {
			return_state: BehaviorState::Failure,
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

		let mut tree = factory.create_from_text(FORCE_STATE)?;
		drop(factory);

		// case 1
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Skipped);
		// case 2
		set_values(&mut tree, BehaviorState::Success, BehaviorState::Failure);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		// case 2
		set_values(&mut tree, BehaviorState::Success, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		// case 3
		set_values(&mut tree, BehaviorState::Failure, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 4
		set_values(&mut tree, BehaviorState::Failure, BehaviorState::Failure);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);

		Ok(())
	}

	const INVERTER: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Inverter name="root_inverter">
			<Action	ID="Action" name="action"/>
		</Inverter>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn inverter() -> Result<(), Error> {
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
			return_state: BehaviorState::Failure,
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

		let mut tree = factory.create_from_text(INVERTER)?;
		drop(factory);

		// case 1
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		// case 2
		set_values(&mut tree, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 3
		set_values(&mut tree, BehaviorState::Running);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Running);
		// case 4
		set_values(&mut tree, BehaviorState::Skipped);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Skipped);

		Ok(())
	}

	const KEEP_RUNNING_UNTIL_FAILURE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<KeepRunningUntilFailure name="keep_running_until_failure">
			<Action	ID="Action" name="action"/>
		</KeepRunningUntilFailure>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn keep_running_until_failure() -> Result<(), Error> {
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

		let mut tree = factory.create_from_text(KEEP_RUNNING_UNTIL_FAILURE)?;
		drop(factory);

		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Running);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Running);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Running);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Running);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Running);
		set_values(&mut tree, BehaviorState::Failure);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);

		Ok(())
	}

	const LOOP_TREE_DEFINITION: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="MainTree">
		<Sequence>
			<Script code="result:=''" />
			<LoopString queue="{queue}"  value="{text}">
				<Script code="result += text + ' '" />
			</LoopString>
		</Sequence>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn loop_queue() -> Result<(), Error> {
		let mut factory = BehaviorTreeFactory::new()?;

		factory.register_behavior_tree_from_text(LOOP_TREE_DEFINITION)?;

		let queue = SharedQueue::<String>::default();
		queue.push_back(String::from("World"));
		queue.push_back(String::from("!"));
		queue.push_front(String::from("Hello"));

		let root_blackboard = Databoard::new();
		root_blackboard.set("queue", queue)?;
		let mut tree = factory.create_tree_with("MainTree", &root_blackboard)?;
		drop(factory);

		let res = tree.tick_while_running().await?;
		assert_eq!(res, BehaviorState::Success);
		assert_eq!(root_blackboard.get::<String>("text")?, String::from("!"));
		assert_eq!(root_blackboard.get::<String>("result")?, String::from("Hello World ! "));

		Ok(())
	}

	const PRECONDITION_XML: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="MainTree">
		<Sequence>
			<Precondition if="value == 42" else="FAILURE">
				<AlwaysSuccess/>
			</Precondition>
			<Precondition if="value != 42" else="SUCCESS">
				<AlwaysFailure/>
			</Precondition>
			<Precondition if="message == 'hello'" else="FAILURE">
				<AlwaysSuccess/>
			</Precondition>
			<Precondition if="message != 'hello'" else="SUCCESS">
				<AlwaysFailure/>
			</Precondition>
		</Sequence>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn precondition() -> Result<(), Error> {
		let mut factory = BehaviorTreeFactory::new()?;

		factory.register_behavior_tree_from_text(PRECONDITION_XML)?;

		let root_blackboard = Databoard::new();
		let mut tree = factory.create_tree_with("MainTree", &root_blackboard)?;
		drop(factory);

		tree.blackboard().set::<i32>("value", 42)?;
		tree.blackboard()
			.set::<String>("message", String::from("hello"))?;
		let result = tree.tick_while_running().await?;
		assert_eq!(result, BehaviorState::Success);

		Ok(())
	}

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

	#[test]
	async fn repeat() -> Result<(), Error> {
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

	const RETRY_UNTIL_SUCCESSFUL: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<RetryUntilSuccessful name="root_retry_until_successful" num_attempts="{num_attempts}">
			<Action	ID="Action" name="action"/>
		</RetryUntilSuccessful>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn retry_until_successful() -> Result<(), Error> {
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
			return_state: BehaviorState::Failure,
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

		let mut tree = factory.create_from_text(RETRY_UNTIL_SUCCESSFUL)?;
		drop(factory);

		tree.blackboard().set("num_attempts", 3)?;
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);

		tree.reset()?;
		tree.blackboard().set("num_attempts", 2)?;
		set_values(&mut tree, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);

		Ok(())
	}

	const RUN_ONCE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<RunOnce name="root_run_once" then_skip="{=}">
			<Action	ID="Action" name="action"/>
		</RunOnce>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn run_once() -> Result<(), Error> {
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
			return_state: BehaviorState::Failure,
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

		let mut tree = factory.create_from_text(RUN_ONCE)?;
		drop(factory);

		// case 1
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Skipped);
		// case 2
		tree.reset()?;
		set_values(&mut tree, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Skipped);
		// case 3
		tree.blackboard().set("then_skip", false)?;
		tree.reset()?;
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		// case 4
		tree.reset()?;
		set_values(&mut tree, BehaviorState::Failure);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);

		Ok(())
	}
}
