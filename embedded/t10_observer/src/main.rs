// Copyright © 2025 Stephan Kunz
//! Embedded version of [t10_observer](examples/t10_observer.rs).

#![no_main]
#![no_std]

use ariel_os::debug::{ExitCode, exit, log::*};
use behaviortree::{BehaviorTreeObserver, prelude::*};

const XML: &str = r#"
<root BTCPP_format="4">
    <BehaviorTree ID="MainTree">
        <Sequence>
            <Fallback>
                <AlwaysFailure name="failing_action"/>
                <SubTree ID="SubTreeA" name="mysub"/>
            </Fallback>
            <AlwaysSuccess name="last_action"/>
        </Sequence>
    </BehaviorTree>

    <BehaviorTree ID="SubTreeA">
        <Sequence>
            <AlwaysSuccess name="action_subA"/>
            <SubTree ID="SubTreeB" name="sub_nested"/>
            <SubTree ID="SubTreeB" />
        </Sequence>
    </BehaviorTree>

    <BehaviorTree ID="SubTreeB">
        <AlwaysSuccess name="action_subB"/>
    </BehaviorTree>

</root>
"#;

#[allow(clippy::expect_used)]
async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::new()?;

	factory.register_behavior_tree_from_text(XML)?;

	let mut tree = factory.create_tree("MainTree")?;
	drop(factory);

	// add the observer
	let observer = BehaviorTreeObserver::new(&mut tree);

	// print tree structure
	// tree.print()?;
	// println!();

	// Print the unique ID and the corresponding human readable path
	// Path is also expected to be unique.
	for element in tree.iter() {
		info!("{} <-> {}", element.uid(), element.groot2_path().as_ref());
	}
	info!("");

	let result = tree.tick_while_running().await?;

	// print statistics
	for item in tree.iter() {
		let stats = observer
			.get_statistics(item.uid())
			.expect("should be there");
		info!(
			"[{}]  T/S/F: {}/{}/{}",
			item.groot2_path().as_ref(),
			stats.transitions_count,
			stats.success_count,
			stats.failure_count
		);
	}
	info!("");

	Ok(result)
}

#[ariel_os::task(autostart)]
async fn main() {
	info!("running t10_observer...");
	match example().await {
		Ok(_) => {
			info!("...succeeded!");
			exit(ExitCode::SUCCESS)
		}
		Err(err) => {
			error!("...failed!");
			error!("{}", err.to_string().as_str());
			exit(ExitCode::FAILURE)
		}
	};
}
