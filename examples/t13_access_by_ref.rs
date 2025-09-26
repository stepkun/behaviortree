// Copyright © 2025 Stephan Kunz
//! Implements the thirteenth tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev).
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-advanced/tutorial_13_blackboard_reference).
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t13_access_by_ref.cpp).

use core::{
	fmt::{Display, Formatter},
	num::ParseIntError,
};

use behaviortree::prelude::*;

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
		write!(f, "({},{})", self.x, self.y)
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
			write!(f, "{point}")?;
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
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		println!("creating PointCloud");
		// put a PointCloud into blackboard
		let p_cloud: PointCloud = PointCloud {
			points: vec![
				Point { x: 0, y: 0 },
				Point { x: 1, y: 1 },
				Point { x: 2, y: 2 },
				Point { x: 3, y: 3 },
			],
		};

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
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		println!("reading PointCloud by reference");
		let p_cloud = behavior.get_ref::<PointCloud>("cloud")?;
		println!("PointCloud is {}", &*p_cloud);
		drop(p_cloud);

		println!("modifying PointCloud by reference");
		let mut p_cloud = behavior.get_mut_ref::<PointCloud>("cloud")?;
		p_cloud.points.push(Point { x: 4, y: 4 });
		println!("PointCloud is now {}", &*p_cloud);

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

#[tokio::main]
async fn main() -> Result<(), Error> {
	example().await?;
	Ok(())
}

#[cfg(test)]
mod test {
	use super::*;

	#[tokio::test]
	async fn t13_access_by_ref() -> Result<(), Error> {
		let result = example().await?;
		assert_eq!(result, BehaviorState::Success);
		Ok(())
	}
}
