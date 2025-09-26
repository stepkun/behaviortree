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
			SharedQueue,
			action::Script,
			decorator::{Loop, Precondition},
		},
		prelude::*,
	};

	#[test]
	async fn entry_updated() -> Result<(), Error> {
		Ok(())
	}

	#[test]
	async fn force_state() -> Result<(), Error> {
		Ok(())
	}

	#[test]
	async fn inverter() -> Result<(), Error> {
		Ok(())
	}

	#[test]
	async fn keep_running_until_failure() -> Result<(), Error> {
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
		register_behavior!(factory, Loop<String>, "LoopString")?;
		register_behavior!(factory, Script, "Script")?;

		factory.register_behavior_tree_from_text(LOOP_TREE_DEFINITION)?;

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
		factory.register_test_behaviors()?;
		register_behavior!(factory, Precondition, "Precondition")?;

		factory.register_behavior_tree_from_text(PRECONDITION_XML)?;

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

	#[test]
	async fn repeat() -> Result<(), Error> {
		Ok(())
	}

	#[test]
	async fn retry_until_successful() -> Result<(), Error> {
		Ok(())
	}

	#[test]
	async fn run_once() -> Result<(), Error> {
		Ok(())
	}
}
