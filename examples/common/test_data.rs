// Copyright © 2025 Stephan Kunz
#![allow(clippy::unnecessary_wraps)]
#![allow(unused)]

//! Test behaviors

extern crate alloc;

// region:		--- modules
use alloc::str::FromStr;
use core::num::ParseFloatError;
use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use behaviortree::prelude::*;
// endregion:	--- modules

/// Behavior `ApproachObject`
/// Example of custom `ActionNode` (synchronous action) without ports.
#[derive(Behavior, Debug, Default)]
pub struct ApproachObject {}

#[async_trait::async_trait]
impl BehaviorInstance for ApproachObject {
    async fn tick(
        &mut self,
        _behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> BehaviorResult {
        println!("ApproachObject: approach_object");
        Ok(BehaviorState::Success)
    }
}

impl BehaviorStatic for ApproachObject {
    fn kind() -> BehaviorKind {
        BehaviorKind::Action
    }
}

/// Function for behavior `CheckBattery`
/// # Errors
/// In this case never :-)
pub fn check_battery() -> BehaviorResult {
    println!("[ Battery: OK ]");
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
        println!("GripperInterface::open");
        self.open = true;
        Ok(BehaviorState::Success)
    }
    /// Close the gripper.
    /// # Errors
    /// In this case never :-)
    pub fn close(&mut self) -> BehaviorResult {
        println!("GripperInterface::close");
        self.open = false;
        Ok(BehaviorState::Success)
    }
}
/// Behavior `SaySomething`
/// Example of custom `ActionNode` (synchronous action) with an input port.
#[derive(Behavior, Debug, Default)]
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
        println!("Robot says: {msg}");
        Ok(BehaviorState::Success)
    }
}

impl BehaviorStatic for SaySomething {
    fn kind() -> BehaviorKind {
        BehaviorKind::Action
    }

    fn provided_ports() -> PortList {
        port_list! {input_port!(String, "message")}
    }
}

/// Behavior `ThinkWhatToSay`
#[derive(Behavior, Debug, Default)]
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
    fn kind() -> BehaviorKind {
        BehaviorKind::Action
    }

    fn provided_ports() -> PortList {
        port_list![output_port!(String, "text")]
    }
}

/// Same as struct `SaySomething`, but to be registered with `SimpleBehavior`
/// # Errors
#[allow(clippy::needless_pass_by_ref_mut)]
pub fn say_something_simple(behavior: &mut BehaviorData) -> BehaviorResult {
    let msg = behavior.get::<String>("message")?;
    println!("Robot2 says: {msg}");
    Ok(BehaviorState::Success)
}

/// `Position2D`
#[derive(Clone, Debug, Default)]
pub struct Position2D {
    pub x: f64,
    pub y: f64,
}

impl FromStr for Position2D {
    type Err = ParseFloatError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        println!("Converting string: \"{value}\"");
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

impl Display for Position2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

/// Behavior `CalculateGoal`
#[derive(Behavior, Debug, Default)]
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
    fn kind() -> BehaviorKind {
        BehaviorKind::Action
    }

    fn provided_ports() -> PortList {
        port_list![output_port!(Position2D, "goal")]
    }
}

/// Behavior `PrintTarget`
#[derive(Behavior, Debug, Default)]
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
        println!("Target positions: [ {}, {} ]", pos.x, pos.y);
        Ok(BehaviorState::Success)
    }
}

impl BehaviorStatic for PrintTarget {
    fn kind() -> BehaviorKind {
        BehaviorKind::Action
    }

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
    type Err = ParseFloatError;

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

impl Display for Pose2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{};{};{}", self.x, self.y, self.theta)
    }
}

/// Behavior `MoveBase`
#[derive(Behavior, Debug)]
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
        println!(
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
            println!("[ MoveBase: FINISHED ]");
            return Ok(BehaviorState::Success);
        }

        Ok(BehaviorState::Running)
    }
}

impl BehaviorStatic for MoveBaseAction {
    fn kind() -> BehaviorKind {
        BehaviorKind::Action
    }

    fn provided_ports() -> PortList {
        port_list![input_port!(Pose2D, "goal")]
    }
}
