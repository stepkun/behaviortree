// Copyright © 2025 Stephan Kunz

//! This test implements the second tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev)
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-basics/tutorial_02_basic_ports)
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t02_basic_ports.cpp)
//!

#[doc(hidden)]
extern crate alloc;
mod common;

use behaviortree::prelude::*;
use common::test_data::{SaySomething, ThinkWhatToSay, say_something_simple};

const XML: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Sequence name="root">
			<SaySomething     message="hello" />
			<SaySomething2    message="this works too" />
			<ThinkWhatToSay   text="{the_answer}"/>
			<SaySomething     message="{the_answer}" />
			<SaySomething2    message="{the_answer}" />
		</Sequence>
	</BehaviorTree>
</root>
"#;

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::default();

	// The struct SaySomething has a method called ports() that defines the INPUTS.
	// In this case, it requires an input called "message"
	register_behavior!(factory, SaySomething, "SaySomething")?;

	// Similarly to SaySomething, ThinkWhatToSay has an OUTPUT port called "text"
	// Both these ports are of type `String`, therefore they can connect to each other
	register_behavior!(factory, ThinkWhatToSay, "ThinkWhatToSay")?;

	// `SimpleBehavior` can not define their own method provided_ports(), therefore
	// we have to pass the PortsList explicitly if we want the Action to use get_input()
	// or set_output();
	let say_something_ports = port_list! {input_port!(String, "message")};
	register_behavior!(
		factory,
		say_something_simple,
		"SaySomething2",
		say_something_ports,
		BehaviorKind::Action
	)?;

	let mut tree = factory.create_from_text(XML)?;
	// dropping the factory to free memory
	drop(factory);

	let result = tree.tick_while_running().await?;
	Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	example().await?;
	Ok(())
}

#[cfg(test)]
mod test {
	use super::*;

	#[tokio::test]
	async fn t02_basic_ports() -> Result<(), Error> {
		let result = example().await?;
		assert_eq!(result, BehaviorState::Success);
		Ok(())
	}
}
