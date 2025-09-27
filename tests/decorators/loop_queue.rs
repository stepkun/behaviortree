// Copyright © 2025 Stephan Kunz
//! Tests the [`LoopQueue`] decorator.

extern crate alloc;

use behaviortree::{
	behavior::{SharedQueue, action::Script, decorator::Loop},
	prelude::*,
};

const TREE_DEFINITION: &str = r#"
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

#[tokio::test]
async fn loop_over_string_queue() -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	register_behavior!(factory, Loop<String>, "LoopString")?;
	register_behavior!(factory, Script, "Script")?;

	factory.register_behavior_tree_from_text(TREE_DEFINITION)?;

	let queue = SharedQueue::<String>::default();
	queue.push_back(String::from("World"));
	queue.push_back(String::from("!"));
	queue.push_front(String::from("Hello"));

	let root_blackboard = Databoard::new();
	root_blackboard.set("queue", queue)?;
	let mut tree = factory.create_tree_with("MainTree", root_blackboard.clone())?;
	drop(factory);

	let res = tree.tick_while_running().await?;
	assert_eq!(res, BehaviorState::Success);
	assert_eq!(root_blackboard.get::<String>("text")?, String::from("!"));
	assert_eq!(root_blackboard.get::<String>("result")?, String::from("Hello World ! "));

	Ok(())
}
