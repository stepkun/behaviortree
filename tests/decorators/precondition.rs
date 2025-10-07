// Copyright © 2025 Stephan Kunz
//! Tests the [`Precondition`] decorator.

extern crate alloc;

use behaviortree::prelude::*;

const XML: &str = r#"
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

#[tokio::test]
async fn precondition() -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::new()?;

	factory.register_behavior_tree_from_text(XML)?;

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
