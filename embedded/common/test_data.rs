// Copyright © 2025 Stephan Kunz
#![allow(clippy::unnecessary_wraps)]
#![allow(unused)]

//! Test behaviors

// region:		--- modules
use ariel_os::{
	debug::log::*,
	time::{Duration, Instant, Timer},
};
use behaviortree::prelude::*;
// endregion:	--- modules

/// Action `ApproachObject`
/// Example of custom `ActionNode` (synchronous action) without ports.
#[derive(Action, Debug, Default)]
pub struct ApproachObject {}

#[async_trait::async_trait]
impl BehaviorInstance for ApproachObject {
	async fn tick(
		&mut self,
		_behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		info!("ApproachObject: approach_object");
		Ok(BehaviorState::Success)
	}
}

impl BehaviorStatic for ApproachObject {}

/// Function for condition `CheckBattery`
/// # Errors
/// In this case never :-)
pub fn check_battery() -> BehaviorResult {
	info!("[ Battery: OK ]");
	Ok(BehaviorState::Success)
}

/// Struct for behaviors `OpenGripper` and `CloseGripper`
#[derive(Default)]
pub struct GripperInterface {
	open: bool,
}

impl GripperInterface {
	/// Open the gripper.
	/// # Errors
	/// In this case never :-)
	pub fn open(&mut self) -> BehaviorResult {
		info!("GripperInterface::open");
		self.open = true;
		Ok(BehaviorState::Success)
	}
	/// Close the gripper.
	/// # Errors
	/// In this case never :-)
	pub fn close(&mut self) -> BehaviorResult {
		info!("GripperInterface::close");
		self.open = false;
		Ok(BehaviorState::Success)
	}
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

/// Action `ThinkWhatToSay`
#[derive(Action, Debug, Default)]
pub struct ThinkWhatToSay {}

#[async_trait::async_trait]
impl BehaviorInstance for ThinkWhatToSay {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		behavior.set("text", String::from("The answer is 42"))?;
		Ok(BehaviorState::Success)
	}
}

impl BehaviorStatic for ThinkWhatToSay {
	fn provided_ports() -> PortList {
		port_list![output_port!(String, "text")]
	}
}

/// Same as struct `SaySomething`, but to be registered with `SimpleBehavior`
/// # Errors
#[allow(clippy::needless_pass_by_ref_mut)]
pub fn say_something_simple(behavior: &mut BehaviorData) -> BehaviorResult {
	let msg = behavior.get::<String>("message")?;
	info!("Robot2 says: {}", msg.as_str());
	Ok(BehaviorState::Success)
}

/// `Position2D`
#[derive(Clone, Debug, Default)]
pub struct Position2D {
	pub x: f64,
	pub y: f64,
}

impl FromStr for Position2D {
	type Err = core::num::ParseFloatError;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		info!("Converting string: \"{}\"", value);
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
		Ok(Self { x, y })
	}
}

impl core::fmt::Display for Position2D {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "[{}, {}]", self.x, self.y)
	}
}

/// Action `CalculateGoal`
#[derive(Action, Debug, Default)]
pub struct CalculateGoal {}

#[async_trait::async_trait]
impl BehaviorInstance for CalculateGoal {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let mygoal = Position2D { x: 1.1, y: 2.3 };
		behavior.set("goal", mygoal)?;
		Ok(BehaviorState::Success)
	}
}

impl BehaviorStatic for CalculateGoal {
	fn provided_ports() -> PortList {
		port_list![output_port!(Position2D, "goal")]
	}
}

/// Action `PrintTarget`
#[derive(Action, Debug, Default)]
pub struct PrintTarget {}

#[async_trait::async_trait]
impl BehaviorInstance for PrintTarget {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let pos = behavior.get::<Position2D>("target")?;
		info!("Target positions: [ {}, {} ]", pos.x, pos.y);
		Ok(BehaviorState::Success)
	}
}

impl BehaviorStatic for PrintTarget {
	fn provided_ports() -> PortList {
		port_list![input_port!(Position2D, "target")]
	}
}

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
