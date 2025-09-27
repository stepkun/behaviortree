// Copyright © 2025 Stephan Kunz
//! Tests the [`Script`] action.

extern crate alloc;

use behaviortree::{behavior::action::Script, prelude::*};

const XML: &str = r#"
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

#[tokio::test]
async fn script() -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, Script, "Script")?;

	factory.register_behavior_tree_from_text(XML)?;

	let root_blackboard = Databoard::new();
	let mut tree = factory.create_tree_with("MainTree", root_blackboard.clone())?;
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
