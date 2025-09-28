// Copyright © 2025 Stephan Kunz
//! Tests the [`SetBlackboard`] & [`UnsetBlackboard`] actions.

extern crate alloc;

use behaviortree::{
	behavior::action::{SetBlackboard, UnsetBlackboard},
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

#[tokio::test]
async fn set_unset_blackboard() -> Result<(), Error> {
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
