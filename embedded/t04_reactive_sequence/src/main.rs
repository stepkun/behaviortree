// Copyright © 2025 Stephan Kunz
//! Embedded version of [t04_reactive_sequence](examples/t04_reactive_sequence.rs).

#![no_main]
#![no_std]

#[path = "../../common/mod.rs"]
mod common;

use ariel_os::{
	debug::{ExitCode, exit, log::*},
	time::Timer,
};
use behaviortree::prelude::*;
use common::test_data::{MoveBaseAction, SaySomething, check_battery};

const XML: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
        <Sequence name="std root sequence">
            <BatteryOK/>
            <SaySomething   message="mission started..." />
            <MoveBase       goal="1;2;3"/>
            <SaySomething   message="mission completed!" />
        </Sequence>
	</BehaviorTree>
</root>
"#;

const XML_REACTIVE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="ReactiveMainTree">
	<BehaviorTree ID="ReactiveMainTree">
		<ReactiveSequence name="reactive root sequence">
            <BatteryOK/>
            <Sequence name = "inner std sequence">
                <SaySomething   message="mission started..." />
                <MoveBase       goal="1;2;3"/>
                <SaySomething   message="mission completed!" />
            </Sequence>
		</ReactiveSequence>
	</BehaviorTree>
</root>
"#;

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_core_behaviors()?;

	register_behavior!(factory, check_battery, "BatteryOK", BehaviorKind::Condition)?;
	register_behavior!(factory, MoveBaseAction, "MoveBase")?;
	register_behavior!(factory, SaySomething, "SaySomething")?;

	let mut tree = factory.create_from_text(XML)?;
	let mut reactive_tree = factory.create_from_text(XML_REACTIVE)?;
	drop(factory);

	info!("=> Running BT with std sequence:");
	let mut result = tree.tick_once().await?;
	while result == BehaviorState::Running {
		Timer::after_millis(100).await;
		result = tree.tick_once().await?;
	}

	// run the reactive BT using own loop with sleep to avoid busy loop
	info!("\n\n=> Running BT with reactive sequence:");
	let mut result = reactive_tree.tick_once().await?;
	while result == BehaviorState::Running {
		Timer::after_millis(100).await;
		result = reactive_tree.tick_once().await?;
	}
	Ok(result)
}

#[ariel_os::task(autostart)]
async fn main() {
	info!("running t04_reactive_sequence...");
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
