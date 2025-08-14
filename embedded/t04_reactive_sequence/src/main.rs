// Copyright © 2025 Stephan Kunz
#![no_main]
#![no_std]

//! Embedded version of [t04_reactive_sequence](examples/t04_reactive_sequence.rs)

use ariel_os::{
	debug::{ExitCode, exit, log::*},
	time::{Duration, Instant, Timer},
};
use behaviortree::prelude::*;

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

/// `Position2D`
#[derive(Clone, Debug, Default)]
pub struct Pose2D {
	/// x
	pub x: f64,
	/// y
	pub y: f64,
	/// rotation
	pub theta: f64,
}

impl FromStr for Pose2D {
	type Err = core::num::ParseFloatError;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		// remove redundant ' and &apos; from string
		let s = value
			.replace('\'', "")
			.trim()
			.replace("&apos;", "")
			.trim()
			.to_string();
		let v: Vec<&str> = s.split(';').collect();
		let x = f64::from_str(v[0])?;
		let y = f64::from_str(v[1])?;
		let theta = f64::from_str(v[2])?;
		Ok(Self { x, y, theta })
	}
}

impl core::fmt::Display for Pose2D {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{};{};{}", self.x, self.y, self.theta)
	}
}

/// Function for condition `CheckBattery`
/// # Errors
/// In this case never :-)
pub fn check_battery() -> BehaviorResult {
	info!("[ Battery: OK ]");
	Ok(BehaviorState::Success)
}

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

/// Action `MoveBase`
#[derive(Action, Debug)]
pub struct MoveBaseAction {
	start_time: Instant,
	completion_time: Duration,
}

impl Default for MoveBaseAction {
	fn default() -> Self {
		Self {
			start_time: Instant::now(),
			completion_time: Duration::default(),
		}
	}
}

#[async_trait::async_trait]
impl BehaviorInstance for MoveBaseAction {
	fn on_start(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> Result<(), BehaviorError> {
		let pose = behavior.get::<Pose2D>("goal")?;
		info!(
			"[ MoveBase: SEND REQUEST ]. goal: x={} y={} theta={}",
			pose.x, pose.y, pose.theta
		);
		self.start_time = Instant::now();
		self.completion_time = Duration::from_millis(220);
		Ok(())
	}

	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		if Instant::now().duration_since(self.start_time) >= self.completion_time {
			info!("[ MoveBase: FINISHED ]");
			return Ok(BehaviorState::Success);
		}

		Ok(BehaviorState::Running)
	}
}

impl BehaviorStatic for MoveBaseAction {
	fn provided_ports() -> PortList {
		port_list![input_port!(Pose2D, "goal")]
	}
}

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
