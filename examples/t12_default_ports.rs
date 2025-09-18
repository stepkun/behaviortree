// Copyright © 2025 Stephan Kunz
//! Implements the twelvth tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev).
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-advanced/tutorial_12_default_ports).
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t12_default_ports.cpp).

use behaviortree::prelude::*;
use core::fmt::{Display, Formatter};
use nanoserde::{DeJson, SerJson};

const XML: &str = r#"
<root BTCPP_format="4">
  	<BehaviorTree ID="MainTree">
        <Sequence>
            <NodeWithDefaultPoints input="-1,-2"/>
        </Sequence>
  	</BehaviorTree>
</root>
"#;

// Add derive of DeJson & SerJson for bi-directional conversion to JSON
// using crate 'nanoserde'
#[derive(Clone, Copy, Debug, PartialEq, Eq, DeJson, SerJson)]
struct Point2D {
	x: i32,
	y: i32,
}

impl Display for Point2D {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "{},{}", self.x, self.y)
	}
}

impl FromStr for Point2D {
	type Err = BehaviorError;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		// check for json marker
		if let Some(stripped) = value.strip_prefix("json:") {
			let res = DeJson::deserialize_json(stripped)?;
			Ok(res)
		} else
		// check for json content
		if let Ok(res) = DeJson::deserialize_json(value) {
			Ok(res)
		} else
		// try conventional
		{
			let v: Vec<&str> = value.split(',').collect();
			let x = i32::from_str(v[0])?;
			let y = i32::from_str(v[1])?;
			Ok(Self { x, y })
		}
	}
}

#[derive(Action, Debug, Default)]
struct BehaviorWithDefaultPoints {}

#[async_trait::async_trait]
impl Behavior for BehaviorWithDefaultPoints {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let msg: String = behavior.get("input")?;
		let point = Point2D::from_str(&msg).map_err(|_| BehaviorError::ParsePortValue("input".into(), msg.into()))?;
		assert_eq!(point, Point2D { x: -1, y: -2 });
		println!("input:  [{},{}]", point.x, point.y);

		let point: Point2D = behavior.get("pointA")?;
		assert_eq!(point, Point2D { x: 1, y: 2 });
		println!("pointA:  [{},{}]", point.x, point.y);

		let point: Point2D = behavior.get("pointB")?;
		assert_eq!(point, Point2D { x: 3, y: 4 });
		println!("pointB:  [{},{}]", point.x, point.y);

		let msg: String = behavior.get("pointC")?;
		let point = Point2D::from_str(&msg).map_err(|_| BehaviorError::ParsePortValue("pointC".into(), msg.into()))?;
		assert_eq!(point, Point2D { x: 5, y: 6 });
		println!("pointC:  [{},{}]", point.x, point.y);

		let point: Point2D = behavior.get("pointD")?;
		assert_eq!(point, Point2D { x: 7, y: 8 });
		println!("pointD:  [{},{}]", point.x, point.y);

		let point: Point2D = behavior.get("pointE")?;
		assert_eq!(point, Point2D { x: 9, y: 10 });
		println!("pointE:  [{},{}]", point.x, point.y);

		let point: Point2D = behavior.get("pointF")?;
		assert_eq!(point, Point2D { x: 11, y: 12 });
		println!("pointF:  [{},{}]", point.x, point.y);

		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list!(
			input_port!(String, "input"),                             // default value from XML is [-1,-2]
			input_port!(Point2D, "pointA", Point2D { x: 1, y: 2 }),   // default value is [1,2]
			input_port!(Point2D, "pointB", "{point}"),                // default value inside blackboard {pointB}
			input_port!(Point2D, "pointC", "5,6"),                    // default value is [5,6],
			input_port!(Point2D, "pointD", "{=}"),                    // default value inside blackboard {pointD}
			input_port!(Point2D, "pointE", r#"json:{"x":9,"y":10}"#), // default value is [9,10]
			input_port!(Point2D, "pointF", r#"{"x":11,"y":12}"#),     // default value is [11,12]
		)
	}
}

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_core_behaviors()?;

	register_behavior!(factory, BehaviorWithDefaultPoints, "NodeWithDefaultPoints")?;

	factory.register_behavior_tree_from_text(XML)?;

	let mut tree = factory.create_tree("MainTree")?;
	drop(factory);

	// initialize blackboard values
	tree.blackboard()
		.set("point", Point2D { x: 3, y: 4 })?;
	tree.blackboard()
		.set("pointD", Point2D { x: 7, y: 8 })?;

	// run the BT
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
	async fn t12_default_ports() -> Result<(), Error> {
		let result = example().await?;
		assert_eq!(result, BehaviorState::Success);
		Ok(())
	}
}
