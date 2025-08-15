// Copyright © 2025 Stephan Kunz
#![no_main]
#![no_std]

//! Embedded version of [t06_subtree_port_remappings](examples/t06_subtree_port_remappings.rs).

#[path = "../../common/mod.rs"]
mod common;

use ariel_os::debug::{ExitCode, exit, log::*};
use behaviortree::prelude::*;
use common::test_data::{MoveBaseAction, SaySomething};

const XML: &str = r#"
<root BTCPP_format="4">
    <BehaviorTree ID="MainTree">
        <Sequence>
            <Script code=" move_goal:='1;2;3' " />
            <SubTree ID="MoveRobot" target="{move_goal}" result="{move_result}" />
            <SaySomething message="{move_result}"/>
        </Sequence>
    </BehaviorTree>

    <BehaviorTree ID="MoveRobot">
        <Fallback>
            <Sequence>
                <MoveBase  goal="{target}"/>
                <Script code=" result:='goal reached' " />
            </Sequence>
            <ForceFailure>
                <Script code=" result:='error' " />
            </ForceFailure>
        </Fallback>
    </BehaviorTree>
</root>
"#;

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_groot2_behaviors()?;

	register_behavior!(factory, SaySomething, "SaySomething")?;
	register_behavior!(factory, MoveBaseAction, "MoveBase")?;

	factory.register_behavior_tree_from_text(XML)?;
	let mut tree = factory.create_main_tree()?;
	drop(factory);

	let result = tree.tick_while_running().await?;

	info!("------ Root BB ------");
	tree.subtree(0)?.blackboard().debug_message();
	info!("----- Second BB -----");
	tree.subtree(1)?.blackboard().debug_message();
	Ok(result)
}

#[ariel_os::task(autostart)]
async fn main() {
	info!("running t06_subtree_port_remappings...");
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
