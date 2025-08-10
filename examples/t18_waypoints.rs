// Copyright © 2025 Stephan Kunz

//! This test implements the eighteenth tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev)
//!
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t18_waypoints.cpp)
//!

// //! [tutorial:](https://www.behaviortree.dev/docs/tutorial-advanced/tutorial_18_XXX)

extern crate alloc;
mod common;

use behaviortree::{behavior::decorator::{Loop, SharedQueue}, prelude::*};
use common::test_data::Pose2D;
use std::time::Duration;

#[derive(Behavior, Debug, Default)]
struct GenerateWaypoints;

#[async_trait::async_trait]
impl BehaviorInstance for GenerateWaypoints {
    async fn tick(
        &mut self,
        behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
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
}

impl BehaviorStatic for GenerateWaypoints {
    fn kind() -> BehaviorKind {
        BehaviorKind::Action
    }

    fn provided_ports() -> PortList {
        port_list![output_port!(SharedQueue<Pose2D>, "waypoints"),]
    }
}

#[derive(Behavior, Debug, Default)]
struct PrintNumber;

#[async_trait::async_trait]
impl BehaviorInstance for PrintNumber {
    async fn tick(
        &mut self,
        behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> BehaviorResult {
        let value: f64 = behavior.get("value")?;
        println!("PrintNumber: {}", value);

        Ok(BehaviorState::Success)
    }
}

impl BehaviorStatic for PrintNumber {
    fn kind() -> BehaviorKind {
        BehaviorKind::Action
    }

    fn provided_ports() -> PortList {
        port_list![input_port!(f64, "value"),]
    }
}

#[derive(Behavior, Debug, Default)]
struct UseWaypoint;

#[async_trait::async_trait]
impl BehaviorInstance for UseWaypoint {
    async fn tick(
        &mut self,
        behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> BehaviorResult {
        if let Ok(wp) = behavior.get::<Pose2D>("waypoint") {
            tokio::time::sleep(Duration::from_millis(100)).await;
            println!("Using waypoint: {}/{}", wp.x, wp.y);
            Ok(BehaviorState::Success)
        } else {
            Ok(BehaviorState::Failure)
        }
    }
}

impl BehaviorStatic for UseWaypoint {
    fn kind() -> BehaviorKind {
        BehaviorKind::Action
    }

    fn provided_ports() -> PortList {
        port_list![input_port!(Pose2D, "waypoint",),]
    }
}

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

async fn example() -> BehaviorTreeResult {
    let mut factory = BehaviorTreeFactory::with_groot2_behaviors()?;

    // @TODO:
    factory.register_behavior_type::<Loop<Pose2D>>("LoopPose")?;
    //register_behavior!(factory, Loop<Pose2D>, "LoopPose")?;

    register_behavior!(factory, UseWaypoint, "UseWaypoint")?;
    register_behavior!(factory, PrintNumber, "PrintNumber")?;
    register_behavior!(factory, GenerateWaypoints, "GenerateWaypoints")?;

    let mut tree = factory.create_from_text(XML)?;
    drop(factory);

    let result = tree.tick_while_running().await?;
    Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    example().await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn t18_waypoints() -> Result<(), Error> {
        let result = example().await?;
        assert_eq!(result, BehaviorState::Success);
        Ok(())
    }
}
