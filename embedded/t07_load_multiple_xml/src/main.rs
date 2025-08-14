// Copyright © 2025 Stephan Kunz
#![no_main]
#![no_std]

//! Embedded version of [t07_load_multiple_xml](examples/t07_load_multiple_xml.rs)

use ariel_os::debug::{ExitCode, exit, log::*};

use behaviortree::prelude::*;

const XML_MAIN: &str = r#"
<root BTCPP_format="4">
    <BehaviorTree ID="MainTree">
        <Sequence>
            <SaySomething message="starting MainTree" />
            <SubTree ID="SubA"/>
            <SubTree ID="SubB"/>
        </Sequence>
    </BehaviorTree>
</root>
"#;

const XML_SUB_A: &str = r#"
<root BTCPP_format="4">
    <BehaviorTree ID="SubA">
        <SaySomething message="Executing SubA" />
    </BehaviorTree>
</root>
"#;

const XML_SUB_B: &str = r#"
<root BTCPP_format="4">
    <BehaviorTree ID="SubB">
        <SaySomething message="Executing SubB" />
    </BehaviorTree>
</root>
"#;

/// Action `SaySomething`
/// Example of custom `ActionNode` (synchronous action) with an input port.
#[derive(Action, Debug, Default)]
pub struct SaySomething {}

#[async_trait::async_trait]
impl BehaviorInstance for SaySomething {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let msg = behavior.get::<String>("message")?;
		info!("Robot says: {}", msg.as_str());
		Ok(BehaviorState::Success)
	}
}

impl BehaviorStatic for SaySomething {
	fn provided_ports() -> PortList {
		port_list! {input_port!(String, "message")}
	}
}

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_groot2_behaviors()?;

	register_behavior!(factory, SaySomething, "SaySomething")?;

	// Register the behavior tree definitions, but do not instantiate them yet.
	// Order is not important.
	factory.register_behavior_tree_from_text(XML_SUB_A)?;
	factory.register_behavior_tree_from_text(XML_SUB_B)?;
	factory.register_behavior_tree_from_text(XML_MAIN)?;

	//Check that the BTs have been registered correctly
	info!("Registered BehaviorTrees:");
	for bt_name in factory.registered_behavior_trees() {
		info!(" - {}", *bt_name);
	}

	// You can create the MainTree and the subtrees will be added automatically.
	let mut tree = factory.create_tree("MainTree")?;
	// ... and/or you can create only one of the subtrees
	let mut sub_tree_a = factory.create_tree("SubA")?;
	drop(factory);

	info!("----- MainTree tick ----");
	let result = tree.tick_while_running().await?;

	info!("----- SubA tick ----");
	sub_tree_a.tick_while_running().await?;
	Ok(result)
}

#[ariel_os::task(autostart)]
async fn main() {
	info!("running t07_load_multiple_xml...");
	match example().await {
		Ok(_) => {
			info!("...succeeded!");
			exit(ExitCode::SUCCESS)
		}
		Err(_) => {
			error!("...failed!");
			exit(ExitCode::FAILURE)
		}
	};
}
