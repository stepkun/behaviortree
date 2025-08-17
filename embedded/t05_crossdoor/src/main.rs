// Copyright © 2025 Stephan Kunz
#![no_main]
#![no_std]

//! Embedded version of [t05_crossdoor](examples/t05_crossdoor.rs).

#[path = "../../common/mod.rs"]
mod common;

use ariel_os::{
	debug::{ExitCode, exit, log::*},
	// time::{Duration, Timer},
};
use behaviortree::prelude::*;
use common::cross_door::CrossDoor;

const XML: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="CrossDoor">
        <Sequence>
            <Fallback>
                <Inverter>
                    <IsDoorClosed/>
                </Inverter>
                <SubTree ID="DoorClosed"/>
            </Fallback>
            <PassThroughDoor/>
        </Sequence>
	</BehaviorTree>

    <BehaviorTree ID="DoorClosed">
        <Fallback>
            <OpenDoor/>
            <RetryUntilSuccessful num_attempts="5">
                <PickLock/>
            </RetryUntilSuccessful>
            <SmashDoor/>
        </Fallback>
    </BehaviorTree>
</root>
"#;

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_core_behaviors()?;

	CrossDoor::register_behaviors(&mut factory)?;

	// In this example a single XML contains multiple <BehaviorTree>
	// To determine which one is the "main one", we should first register
	// the XML and then allocate a specific tree, using its ID
	factory.register_behavior_tree_from_text(XML)?;
	let mut tree = factory.create_tree("CrossDoor")?;
	drop(factory);

	let result = tree.tick_while_running().await?;
	Ok(result)
}

#[ariel_os::task(autostart)]
async fn main() {
	info!("running t05_crossdoor...");
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
