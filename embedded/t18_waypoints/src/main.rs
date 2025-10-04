// Copyright © 2025 Stephan Kunz
//! Embedded version of [t18_waypoints](examples/t18_waypints.rs).

#![no_main]
#![no_std]

#[path = "../../common/mod.rs"]
mod common;

use ariel_os::{
	debug::{ExitCode, exit, log::*},
	time::Timer,
};
use behaviortree::{
	behavior::{SharedQueue, decorator::Loop},
	prelude::*,
};
use common::test_data::Pose2D;

const XML: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="TreeA">
		<Sequence>
			<LoopDouble queue="1;2;3"  value="{number}">
				<PrintNumber value="{number}" />
			</LoopDouble>

			<GenerateWaypoints waypoints="{waypoints}" />
			<LoopPose queue="{waypoints}"  value="{wp}">
				<UseWaypoint waypoint="{wp}" />
			</LoopPose>
		</Sequence>
	</BehaviorTree>
</root>
"#;

#[derive(Action, Debug, Default)]
struct GenerateWaypoints;

#[async_trait::async_trait]
impl Behavior for GenerateWaypoints {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let shared_queue = SharedQueue::default();
		for i in 0..5 {
			shared_queue.push_back(Pose2D {
				x: f64::from(i),
				y: f64::from(i),
				theta: 0_f64,
			});
		}

		behavior.set("waypoints", shared_queue)?;

		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list![output_port!(SharedQueue<Pose2D>, "waypoints"),]
	}
}

#[derive(Action, Debug, Default)]
struct PrintNumber;

#[async_trait::async_trait]
impl Behavior for PrintNumber {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let value: f64 = behavior.get("value")?;
		info!("PrintNumber: {}", value);

		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(f64, "value"),]
	}
}

#[derive(Action, Debug, Default)]
struct UseWaypoint;

#[async_trait::async_trait]
impl Behavior for UseWaypoint {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		if let Ok(wp) = behavior.get::<Pose2D>("waypoint") {
			Timer::after_millis(100).await;
			info!("Using waypoint: {}/{}", wp.x, wp.y);
			Ok(BehaviorState::Success)
		} else {
			Ok(BehaviorState::Failure)
		}
	}

	fn provided_ports() -> PortList {
		port_list![input_port!(Pose2D, "waypoint",),]
	}
}

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_groot2_behaviors()?;

	factory.register_behavior_type::<Loop<Pose2D>>("LoopPose")?;

	register_behavior!(factory, UseWaypoint, "UseWaypoint")?;
	register_behavior!(factory, PrintNumber, "PrintNumber")?;
	register_behavior!(factory, GenerateWaypoints, "GenerateWaypoints")?;

	let mut tree = factory.create_from_text(XML)?;
	drop(factory);

	let result = tree.tick_while_running().await?;
	Ok(result)
}

#[ariel_os::task(autostart)]
async fn main() {
	info!("running t18_waypoints...");
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
