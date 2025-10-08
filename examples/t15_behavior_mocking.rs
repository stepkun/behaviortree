// Copyright © 2025 Stephan Kunz
//! Implements the fifteenth tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev).
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-advanced/tutorial_15_replace_rules).
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t15_nodes_mocking.cpp).

#[path = "./common/test_data.rs"]
mod test_data;

use behaviortree::{behavior::mock_behavior::MockBehaviorConfig, factory::registry::SubstitutionRule, prelude::*};
use core::time::Duration;
use test_data::SaySomething;

const XML: &str = r#"
<root BTCPP_format="4">
  	<BehaviorTree ID="MainTree">
		<Sequence>
			<SaySomething name="talk" message="hello world"/>

			<SubTree ID="MySub" name="mysub"/>

			<Script name="set_message" code="msg:= 'the original message' "/>
			<SaySomething message="{msg}"/>

			<Sequence name="counting">
				<SaySomething message="1"/>
				<SaySomething message="2"/>
				<SaySomething message="3"/>
			</Sequence>
		</Sequence>
  	</BehaviorTree>

	<BehaviorTree ID="MySub">
		<Sequence>
		<AlwaysSuccess name="action_subA"/>
		<AlwaysSuccess name="action_subB"/>
		</Sequence>
	</BehaviorTree>
</root>
"#;

const JSON_CONFIGURATION: &str = r#"{
	"MockBehaviorConfigs": {
		"NewMessage": {
			"async_delay": 2000,
			"return_status": "SUCCESS",
			"post_script": "msg:='message SUBSTITUTED by Config'"
		},
        "NoCounting": {
        	"return_status": "SUCCESS"
        }
	},

	"SubstitutionRules": {
        "mysub/*/action_*": "MockAction",
        "talk": "MockSaySomething",
        "set_message": "NewMessage",
        "counting": "NoCounting"
	}
}"#;

#[allow(clippy::unnecessary_wraps)]
#[allow(clippy::needless_pass_by_ref_mut)]
fn mock_action(behavior: &mut BehaviorData) -> BehaviorResult {
	println!("MockAction substituting behavior with full_path(): {}", behavior.full_path());
	Ok(BehaviorState::Success)
}

#[allow(clippy::unnecessary_wraps)]
#[allow(clippy::needless_pass_by_ref_mut)]
fn mock_say_something(behavior: &mut BehaviorData) -> BehaviorResult {
	let msg = behavior.get::<String>("message")?;
	println!("MockSaySomething: {msg}");
	Ok(BehaviorState::Success)
}

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::new()?;

	register_behavior!(factory, SaySomething, "SaySomething")?;

	factory.register_behavior_tree_from_text(XML)?;

	let mut tree = factory.create_tree("MainTree")?;
	// as a reminder, let's print the full names of all the behaviors
	println!("----- Behaviors fullPath() -------");
	for behavior in tree.iter() {
		println!("{}", behavior.full_path());
	}
	println!("\n------ Output (original) ------");
	let result = tree.tick_while_running().await?;
	assert_eq!(result, BehaviorState::Success);

	// IMPORTANT: all substitutions must be done BEFORE creating the tree
	// During the construction phase of the tree, the substitution
	// rules will be used to instantiate the test behaviors, instead of the
	// original ones, so these behaviors must be registered beforehand.
	register_simple_behavior!(factory, mock_action, "MockAction", PortList::default(), BehaviorKind::Action)?;
	register_simple_behavior!(
		factory,
		mock_say_something,
		"MockSaySomething",
		port_list![input_port!(String, "message")],
		BehaviorKind::Action
	)?;

	// There are 3 mechanisms to create behaviors to be used as 'mocks'.
	//---------------------------------------------------------------
	// Mock type 1: register specific 'mock' behaviors into the factory
	// You can use any type of behavior, from simple functions to full fledged behavior structs.

	// Substitute behaviors which match the wildcard pattern 'mysub/*/action_*'
	// with `MockAction`
	factory.add_substitution_rule("mysub/*/action_*", SubstitutionRule::StringRule("MockAction".into()))?;

	// Substitute the behavior with name 'talk' with `MockSaySomething`
	factory.add_substitution_rule("talk", SubstitutionRule::StringRule("MockSaySomething".into()))?;

	//---------------------------------------------------------------
	// Mock type 2: Use the configurable 'MockBehavior'
	// with a configuration for each substitution rule

	// Substitute the behavior with name 'set_message' with a configured 'MockBehavior'
	// This is the configuration passed to the `MockBehavior` for set_message replacement
	let test_config = MockBehaviorConfig {
		// This will return always SUCCESS
		return_state: BehaviorState::Success,
		// Convert the behavior in asynchronous and wait 2000 ms
		async_delay: Some(Duration::from_millis(2000)),
		// Once completed execute this script
		post_script: Some("msg := 'message SUBSTITUTED' ".into()),
		..Default::default()
	};
	factory.add_substitution_rule("set_message", SubstitutionRule::ConfigRule(test_config))?;

	// You can also substitute entire branches, for instance the sequence 'counting'
	// This is the configuration passed to the 'MockBehavior' for counting replacement
	let counting_config = MockBehaviorConfig {
		// This will return always SUCCESS
		return_state: BehaviorState::Success,
		// This will be synchronous (async_delay is 0 - the default)
		..Default::default()
	};
	factory.add_substitution_rule("counting", SubstitutionRule::ConfigRule(counting_config))?;

	println!("\n------ Output (substituted) ------");
	let mut tree = factory.create_tree("MainTree")?;
	let result = tree.tick_while_running().await?;
	assert_eq!(result, BehaviorState::Success);

	// remove previous substitution rules
	factory.clear_substitution_rules();

	println!("\n------ Output (cleared) ------");
	let mut tree = factory.create_tree("MainTree")?;
	let result = tree.tick_while_running().await?;
	assert_eq!(result, BehaviorState::Success);

	//---------------------------------------------------------------
	// Mock type 3: Use a configuration json (file) with any behavior.
	// In this example we use the 'MockBehavior' with similar configuration
	// as above (see JSON_CONFIGURATION constant)
	factory.load_substitution_rules_from_json(JSON_CONFIGURATION)?;

	println!("\n------ Output (configuration) ------");
	let mut tree = factory.create_tree("MainTree")?;
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
	async fn t15_behavior_mocking() -> Result<(), Error> {
		let result = example().await?;
		assert_eq!(result, BehaviorState::Success);
		Ok(())
	}
}
