// Copyright © 2025 Stephan Kunz
//! Embedded action tests.

#![no_main]
#![no_std]
#![allow(clippy::unwrap_used)]

extern crate alloc;

#[cfg(test)]
#[embedded_test::tests]
mod tests {
	use behaviortree::{
		behavior::{
			SharedQueue,
			action::{PopFromQueue, Script, SetBlackboard, UnsetBlackboard},
			condition::{ScriptCondition, WasEntryUpdated},
		},
		prelude::*,
	};

	const SET_TREE_DEFINITION1: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="SetMainTree1">
		<SetBlackboard output_key="test" value="value1"/>
	</BehaviorTree>
</root>
"#;

	const SET_TREE_DEFINITION2: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="SetMainTree2">
		<SetBlackboard output_key="test" value="value2"/>
	</BehaviorTree>
</root>
"#;

	const UNSET_TREE_DEFINITION: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="UnsetMainTree">
		<UnsetBlackboard key="test" />
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn blackboard() -> Result<(), Error> {
		let mut factory = BehaviorTreeFactory::new()?;
		register_behavior!(factory, SetBlackboard<String>, "SetBlackboard")?;
		register_behavior!(factory, UnsetBlackboard<String>, "UnsetBlackboard")?;

		factory.register_behavior_tree_from_text(SET_TREE_DEFINITION1)?;
		factory.register_behavior_tree_from_text(SET_TREE_DEFINITION2)?;
		factory.register_behavior_tree_from_text(UNSET_TREE_DEFINITION)?;

		let root_blackboard = Databoard::new();
		let mut tree1 = factory.create_tree_with("SetMainTree1", &root_blackboard)?;
		let mut tree2 = factory.create_tree_with("SetMainTree2", &root_blackboard)?;
		let mut tree3 = factory.create_tree_with("UnsetMainTree", &root_blackboard)?;
		drop(factory);

		let val1 = String::from("value1");
		let val2 = String::from("value2");
		assert!(root_blackboard.get::<String>("test").is_err());
		tree1.tick_while_running().await?;
		assert_eq!(tree1.blackboard().get::<String>("test")?, val1);
		assert_eq!(root_blackboard.get::<String>("test")?, val1);
		tree2.tick_while_running().await?;
		assert_eq!(tree2.blackboard().get::<String>("test")?, val2);
		assert_eq!(root_blackboard.get::<String>("test")?, val2);
		tree3.tick_while_running().await?;
		assert!(tree3.blackboard().get::<String>("test").is_err());
		assert!(root_blackboard.get::<String>("test").is_err());

		Ok(())
	}

	const POP_FROM_QUEUE_TREE_DEFINITION: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="MainTree">
		<PopFromQueue/>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn pop_from_queue() -> Result<(), Error> {
		let mut factory = BehaviorTreeFactory::new()?;
		register_behavior!(factory, PopFromQueue<i32>, "PopFromQueue")?;

		factory.register_behavior_tree_from_text(POP_FROM_QUEUE_TREE_DEFINITION)?;

		let queue = SharedQueue::<i32>::default();
		queue.push_back(2);
		queue.push_back(3);
		queue.push_front(1);
		queue.push_back(4);

		let root_blackboard = Databoard::new();
		root_blackboard.set("queue", queue)?;
		let mut tree = factory.create_tree_with("MainTree", &root_blackboard)?;
		drop(factory);

		let mut res = tree.tick_once().await?;
		assert_eq!(res, BehaviorState::Success);
		assert_eq!(root_blackboard.get::<i32>("popped_item")?, 1);
		res = tree.tick_once().await?;
		assert_eq!(res, BehaviorState::Success);
		assert_eq!(root_blackboard.get::<i32>("popped_item")?, 2);
		res = tree.tick_once().await?;
		assert_eq!(res, BehaviorState::Success);
		assert_eq!(root_blackboard.get::<i32>("popped_item")?, 3);
		res = tree.tick_once().await?;
		assert_eq!(res, BehaviorState::Success);
		assert_eq!(root_blackboard.get::<i32>("popped_item")?, 4);
		res = tree.tick_once().await?;
		assert_eq!(res, BehaviorState::Failure);

		Ok(())
	}

	const SCRIPT_XML: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="MainTree">
		<Sequence>
			<Script code="message := 'hello world'"/>
			<Script code=" the_answer := 40 + 2 "/>
			<Script code=" value = the_answer "/>
		</Sequence>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn script() -> Result<(), Error> {
		let mut factory = BehaviorTreeFactory::new()?;
		register_behavior!(factory, Script, "Script")?;

		factory.register_behavior_tree_from_text(SCRIPT_XML)?;

		let root_blackboard = Databoard::new();
		let mut tree = factory.create_tree_with("MainTree", &root_blackboard)?;
		drop(factory);

		tree.blackboard().set::<i32>("value", 24)?;
		assert!(root_blackboard.get::<String>("message").is_err());
		let result = tree.tick_while_running().await?;
		assert_eq!(result, BehaviorState::Success);
		assert_eq!(tree.blackboard().get::<String>("message")?, String::from("hello world"));
		assert_eq!(tree.blackboard().get::<i64>("the_answer")?, 42);
		assert_eq!(tree.blackboard().get::<i32>("value")?, 42);

		Ok(())
	}

	const SCRIPT_CONDITION_XML: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="MainTree">
		<Sequence>
			<Fallback>
				<ScriptCondition code="true"/>
				<AlwaysFailure/>
			</Fallback>
			<Fallback>
				<ScriptCondition code="1 + 1 == 2"/>
				<AlwaysFailure/>
			</Fallback>
			<Fallback>
				<ScriptCondition code="value == 42"/>
				<AlwaysFailure/>
			</Fallback>
			<Fallback>
				<ScriptCondition code="message == 'hello'"/>
				<AlwaysFailure/>
			</Fallback>
		</Sequence>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn script_conditions() -> Result<(), Error> {
		let mut factory = BehaviorTreeFactory::new()?;
		factory.register_test_behaviors()?;
		register_behavior!(factory, ScriptCondition, "ScriptCondition")?;

		factory.register_behavior_tree_from_text(SCRIPT_CONDITION_XML)?;

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

	const WAS_ENTRY_UPDATED_XML: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<WasEntryUpdated name="was_entry_updated" entry="test" />
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn was_entry_updated() -> Result<(), Error> {
		let mut factory = BehaviorTreeFactory::new()?;
		register_behavior!(factory, WasEntryUpdated, "WasEntryUpdated")?;

		let mut tree = factory.create_from_text(WAS_ENTRY_UPDATED_XML)?;
		drop(factory);

		tree.blackboard().set("test", 1)?;
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		tree.blackboard().set("test", 2)?;
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);

		tree.blackboard().delete::<i32>("test")?;
		tree.reset()?;

		tree.blackboard().set("test", 1)?;
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		tree.blackboard().set("test", 2)?;
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);

		Ok(())
	}
}
