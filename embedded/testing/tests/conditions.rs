// Copyright © 2025 Stephan Kunz
//! Embedded condition tests.

#![no_main]
#![no_std]
#![allow(clippy::unwrap_used)]

extern crate alloc;

#[cfg(test)]
#[embedded_test::tests]
mod tests {
	use behaviortree::{
		behavior::condition::{ScriptCondition, WasEntryUpdated},
		prelude::*,
	};

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
		let mut tree = factory.create_tree_with("MainTree", root_blackboard.clone())?;
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
