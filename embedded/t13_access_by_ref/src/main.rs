// Copyright © 2025 Stephan Kunz
//! Embedded version of [t12_default_ports](examples/t12_default_ports.rs).

#![no_main]
#![no_std]

use ariel_os::debug::{ExitCode, exit, log::*};
use behaviortree::prelude::*;
use core::{
	fmt::{Display, Formatter},
	num::ParseIntError,
};

const XML: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="PointCloud">
		<Sequence>
			<CreatePointCloud  cloud="{pointcloud}"/>
 			<ModifyPointCloud  cloud="{pointcloud}"/>
		</Sequence>
	</BehaviorTree>
</root>
"#;

#[derive(Clone, Debug)]
struct Point {
	x: i32,
	y: i32,
}

impl Display for Point {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "{},{}", self.x, self.y)
	}
}

impl FromStr for Point {
	type Err = ParseIntError;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		let v: Vec<&str> = value.split(',').collect();
		let x = i32::from_str(v[0])?;
		let y = i32::from_str(v[1])?;
		Ok(Self { x, y })
	}
}

#[derive(Clone, Debug)]
struct PointCloud {
	points: Vec<Point>,
}

impl Display for PointCloud {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "[")?;
		let mut delimiter = false;
		for point in &self.points {
			if delimiter {
				write!(f, ";")?;
			}
			write!(f, "{}", point)?;
			delimiter = true;
		}
		write!(f, "]")
	}
}

impl FromStr for PointCloud {
	type Err = ParseIntError;

	fn from_str(_value: &str) -> Result<Self, Self::Err> {
		todo!()
	}
}

/// Action `CreatePointCloud`
#[derive(Action, Debug, Default)]
struct CreatePointCloud {}

#[async_trait::async_trait]
impl Behavior for CreatePointCloud {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		info!("creating PointCloud");
		// put a PointCloud into blackboard
		let mut points = Vec::with_capacity(6);
		points.push(Point { x: 0, y: 0 });
		points.push(Point { x: 1, y: 1 });
		points.push(Point { x: 2, y: 2 });
		points.push(Point { x: 3, y: 3 });
		let p_cloud: PointCloud = PointCloud { points };

		behavior.set::<PointCloud>("cloud", p_cloud)?;

		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list!(output_port!(PointCloud, "cloud"))
	}
}

/// Action `ModifyPointCloud`
#[derive(Action, Debug, Default)]
struct ModifyPointCloud();

#[async_trait::async_trait]
impl Behavior for ModifyPointCloud {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		info!("reading PointCloud by reference");
		let p_cloud = behavior.get_ref::<PointCloud>("cloud")?;
		for point in &*p_cloud.points {
			info!("Point is {}, {}", point.x, point.y);
		}
		drop(p_cloud);

		info!("modifying PointCloud by reference");
		let mut p_cloud = behavior.get_mut_ref::<PointCloud>("cloud")?;
		p_cloud.points.push(Point { x: 4, y: 4 });
		for point in &*p_cloud.points {
			info!("Point is {}, {}", point.x, point.y);
		}

		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list!(inout_port!(PointCloud, "cloud"),)
	}
}

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::default();

	register_behavior!(factory, CreatePointCloud, "CreatePointCloud")?;
	register_behavior!(factory, ModifyPointCloud, "ModifyPointCloud")?;

	factory.register_behavior_tree_from_text(XML)?;

	let mut tree = factory.create_tree("PointCloud")?;
	drop(factory);

	let result = tree.tick_while_running().await?;
	Ok(result)
}

#[ariel_os::task(autostart)]
async fn main() {
	info!("running t13_access_by_ref...");
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
