// Copyright © 2025 Stephan Kunz
//! Tests the [`PopFromQueue`] action.

extern crate alloc;

use behaviortree::{
	behavior::{SharedQueue, action::PopFromQueue},
	prelude::*,
};

const TREE_DEFINITION: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="MainTree">
		<PopFromQueue/>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn pop_from_int_queue() -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, PopFromQueue<i32>, "PopFromQueue")?;

	factory.register_behavior_tree_from_text(TREE_DEFINITION)?;

	let queue = SharedQueue::<i32>::default();
	queue.push_back(2);
	queue.push_back(3);
	queue.push_front(1);
	queue.push_back(4);

	let root_blackboard = Databoard::new();
	root_blackboard.set("queue", queue)?;
	let mut tree = factory.create_tree_with("MainTree", root_blackboard.clone())?;
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

#[tokio::test]
#[allow(clippy::float_cmp)]
async fn pop_from_double_queue() -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, PopFromQueue<f64>, "PopFromQueue")?;

	factory.register_behavior_tree_from_text(TREE_DEFINITION)?;

	let queue = SharedQueue::<f64>::default();
	queue.push_back(2.1);
	queue.push_back(3.2);
	queue.push_front(1.0);
	queue.push_back(4.3);

	let root_blackboard = Databoard::new();
	root_blackboard.set("queue", queue)?;
	let mut tree = factory.create_tree_with("MainTree", root_blackboard.clone())?;
	drop(factory);

	let mut res = tree.tick_once().await?;
	assert_eq!(res, BehaviorState::Success);
	assert_eq!(root_blackboard.get::<f64>("popped_item")?, 1.0);
	res = tree.tick_once().await?;
	assert_eq!(res, BehaviorState::Success);
	assert_eq!(root_blackboard.get::<f64>("popped_item")?, 2.1);
	res = tree.tick_once().await?;
	assert_eq!(res, BehaviorState::Success);
	assert_eq!(root_blackboard.get::<f64>("popped_item")?, 3.2);
	res = tree.tick_once().await?;
	assert_eq!(res, BehaviorState::Success);
	assert_eq!(root_blackboard.get::<f64>("popped_item")?, 4.3);
	res = tree.tick_once().await?;
	assert_eq!(res, BehaviorState::Failure);

	Ok(())
}

#[tokio::test]
async fn pop_from_string_queue() -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, PopFromQueue<String>, "PopFromQueue")?;

	factory.register_behavior_tree_from_text(TREE_DEFINITION)?;

	let queue = SharedQueue::<String>::default();
	queue.push_back(String::from("2.1"));
	queue.push_back(String::from("3.2"));
	queue.push_front(String::from("1.0"));
	queue.push_back(String::from("4.3"));

	let root_blackboard = Databoard::new();
	root_blackboard.set("queue", queue)?;
	let mut tree = factory.create_tree_with("MainTree", root_blackboard.clone())?;
	drop(factory);

	let mut res = tree.tick_once().await?;
	assert_eq!(res, BehaviorState::Success);
	assert_eq!(root_blackboard.get::<String>("popped_item")?, String::from("1.0"));
	res = tree.tick_once().await?;
	assert_eq!(res, BehaviorState::Success);
	assert_eq!(root_blackboard.get::<String>("popped_item")?, String::from("2.1"));
	res = tree.tick_once().await?;
	assert_eq!(res, BehaviorState::Success);
	assert_eq!(root_blackboard.get::<String>("popped_item")?, String::from("3.2"));
	res = tree.tick_once().await?;
	assert_eq!(res, BehaviorState::Success);
	assert_eq!(root_blackboard.get::<String>("popped_item")?, String::from("4.3"));
	res = tree.tick_once().await?;
	assert_eq!(res, BehaviorState::Failure);

	Ok(())
}

const TREE_DEFINITION2: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="MainTree">
		<PopFromQueue queue="1;2;3"/>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn pop_from_default_queue() -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, PopFromQueue<i32>, "PopFromQueue")?;

	factory.register_behavior_tree_from_text(TREE_DEFINITION2)?;

	let root_blackboard = Databoard::new();
	let mut tree = factory.create_tree_with("MainTree", root_blackboard.clone())?;
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
	assert_eq!(res, BehaviorState::Failure);

	Ok(())
}
