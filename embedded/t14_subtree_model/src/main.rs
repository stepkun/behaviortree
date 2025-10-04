// Copyright © 2025 Stephan Kunz
//! Embedded version of [t14_subtree_mode](examples/t14_subtree_model.rs).

#![no_main]
#![no_std]

#[path = "../../common/mod.rs"]
mod common;

use ariel_os::debug::{ExitCode, exit, log::*};
use behaviortree::prelude::*;
use common::test_data::SaySomething;

const XML: &str = r#"
<root BTCPP_format="4">
  	<BehaviorTree ID="MainTree">
        <Sequence>
            <Script code="target:='1;2;3'"/>
            <SubTree ID="MoveRobot"
                _autoremap="true"
                frame="world"/>
            <SaySomething message="{result}"/>
        </Sequence>
  	</BehaviorTree>

    <BehaviorTree ID="MoveRobot">
        <Fallback>
            <Sequence>
	            <SaySomething message="{frame}"/>
                <MoveBase goal="{target}"/>
                <Script code="result:='goal_reached'"/>
            </Sequence>
            <ForceFailure>
                <Script code="result:='error'"/>
            </ForceFailure>
        </Fallback>
    </BehaviorTree>
</root>
"#;

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_core_behaviors()?;
	factory.register_test_behaviors()?;

	register_behavior!(factory, SaySomething, "SaySomething")?;
	// register subtrees nodes
	move_robot::register_behaviors(&mut factory)?;

	factory.register_behavior_tree_from_text(XML)?;

	let mut tree = factory.create_tree("MainTree")?;
	drop(factory);

	let result = tree.tick_while_running().await?;
	Ok(result)
}

#[ariel_os::task(autostart)]
async fn main() {
	info!("running t14_subtree_model...");
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

/// Implementation of `MoveRobot` tree
mod move_robot {
	#[allow(clippy::wildcard_imports)]
	use super::*;

	use common::test_data::Pose2D;

	/// Action `MoveBase`
	#[derive(Action, Debug, Default)]
	struct MoveBase {
		counter: usize,
	}

	#[async_trait::async_trait]
	impl Behavior for MoveBase {
		fn on_start(
			&mut self,
			behavior: &mut BehaviorData,
			_children: &mut BehaviorTreeElementList,
			_runtime: &SharedRuntime,
		) -> Result<(), BehaviorError> {
			let pos = behavior.get::<Pose2D>("goal")?;

			info!(
				"[ MoveBase: SEND REQUEST ]. goal: x={} y={} theta={}",
				pos.x, pos.y, pos.theta
			);

			Ok(())
		}

		async fn tick(
			&mut self,
			_behavior: &mut BehaviorData,
			_children: &mut BehaviorTreeElementList,
			_runtime: &SharedRuntime,
		) -> BehaviorResult {
			if self.counter < 5 {
				self.counter += 1;
				info!("--- status: RUNNING");
				Ok(BehaviorState::Running)
			} else {
				info!("[ MoveBase: FINISHED ]");
				Ok(BehaviorState::Success)
			}
		}

		fn provided_ports() -> PortList {
			port_list!(input_port!(Pose2D, "goal"),)
		}
	}

	pub fn register_behaviors(factory: &mut BehaviorTreeFactory) -> Result<(), Error> {
		register_behavior!(factory, MoveBase, "MoveBase")?;
		Ok(())
	}
}
