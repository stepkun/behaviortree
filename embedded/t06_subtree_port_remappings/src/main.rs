// Copyright © 2025 Stephan Kunz
#![no_main]
#![no_std]

//! Embedded version of [t06_subtree_port_remappings](examples/t06_subtree_port_remappings.rs)

use ariel_os::debug::{ExitCode, exit, log::*};

use behaviortree::prelude::*;

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
		info!("Robot says: {msg}");
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
