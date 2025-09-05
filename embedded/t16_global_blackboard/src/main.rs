// Copyright © 2025 Stephan Kunz
//! Embedded version of [t16_global_blackboard](examples/t16_global_blackboard.rs).

#![no_main]
#![no_std]

use ariel_os::debug::{ExitCode, exit, log::*};
use behaviortree::{port::PortRemappings, prelude::*};

const XML: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="MainTree">
		<Sequence>
			<PrintNumber name="main_print" val="{@value}" />
			<SubTree ID="MySub"/>
		</Sequence>
	</BehaviorTree>

	<BehaviorTree ID="MySub">
		<Sequence>
			<PrintNumber name="sub_print" val="{@value}" />
			<Script code="@value_sqr := @value * @value" />
			<SubTree ID="MySubSub"/>
		</Sequence>
	</BehaviorTree>

	<BehaviorTree ID="MySubSub">
        <Sequence>
            <PrintNumber name="sub_sub_print" val="{@value}" />
            <Script code="@value_pow3 := @value * @value * @value" />
            <SubTree ID="MySubSubSub"/>
        </Sequence>
    </BehaviorTree>

    <BehaviorTree ID="MySubSubSub">
        <Sequence>
            <PrintNumber name="sub_sub_sub_print" val="{@value}" />
            <Script code="@value_pow4 := @value * @value * @value * @value" />
        </Sequence>
    </BehaviorTree>
</root>
"#;

/// Action `PrintNumber`
#[derive(Action, Debug, Default)]
struct PrintNumber {}

#[async_trait::async_trait]
impl Behavior for PrintNumber {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let value: i64 = behavior.get("val")?;
		info!("PrintNumber [{}] has val: {}", behavior.description().name().as_ref(), value);

		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list!(input_port!(i64, "val"),)
	}
}

async fn example() -> BehaviorTreeResult {
	// create an external blackboard which will survive the tree
	let mut global_blackboard = SharedBlackboard::default();
	// BT-Trees blackboard has global blackboard as parent
	let root_blackboard =
		SharedBlackboard::with_parent("global", global_blackboard.clone(), PortRemappings::default(), false);

	let mut factory = BehaviorTreeFactory::with_groot2_behaviors()?;

	register_behavior!(factory, PrintNumber, "PrintNumber")?;

	factory.register_behavior_tree_from_text(XML)?;

	let mut tree = factory.create_tree_with("MainTree", root_blackboard)?;
	drop(factory);

	// direct interaction with the global blackboard
	for value in 1..=3 {
		global_blackboard.set("value", value)?;
		let result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);

		let value_sqr = global_blackboard.get::<i64>("@value_sqr")?;
		if value_sqr != value * value {
			return Ok(BehaviorState::Failure);
		}
		info!("[While loop] value: {} value_sqr: {}", value, value_sqr);

		let value_pow3 = global_blackboard.get::<i64>("@value_pow3")?;
		if value_pow3 != value * value * value {
			return Ok(BehaviorState::Failure);
		}

		let value_pow4 = global_blackboard.get::<i64>("@value_pow4")?;
		if value_pow4 != value * value * value * value {
			return Ok(BehaviorState::Failure);
		}
	}

	Ok(BehaviorState::Success)
}

#[ariel_os::task(autostart)]
async fn main() {
	info!("running t16_global_blackboard...");
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
