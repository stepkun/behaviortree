// Copyright © 2025 Stephan Kunz
//! Implements the sixteenth tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev).
//!
//! [tutorial:](https://https://www.behaviortree.dev/docs/tutorial-advanced/tutorial_16_global_blackboard).
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t16_global_blackboard.cpp).

use behaviortree::prelude::*;

const XML: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="MainTree">
		<Sequence>
			<PrintNumber name="main_print" val="{@value}" />
			<SubTree ID="MySub"/>
		</Sequence>
	</BehaviorTree>

	<BehaviorTree ID="MySub">
		<Sequence>
			<PrintNumber name="sub_print" val="{@value}" />
			<Script code="@value_sqr := @value * @value" />
			<SubTree ID="MySubSub"/>
		</Sequence>
	</BehaviorTree>

	<BehaviorTree ID="MySubSub">
        <Sequence>
            <PrintNumber name="sub_sub_print" val="{@value}" />
            <Script code="@value_pow3 := @value * @value * @value" />
            <SubTree ID="MySubSubSub"/>
        </Sequence>
    </BehaviorTree>

    <BehaviorTree ID="MySubSubSub">
        <Sequence>
            <PrintNumber name="sub_sub_sub_print" val="{@value}" />
            <Script code="@value_pow4 := @value * @value * @value * @value" />
        </Sequence>
    </BehaviorTree>
</root>
"#;

/// Action `PrintNumber`
#[derive(Action, Debug, Default)]
struct PrintNumber {}

#[async_trait::async_trait]
impl Behavior for PrintNumber {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let value: i64 = behavior.get("val")?;
		println!("PrintNumber [{}] has val: {value}", behavior.description().name());

		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list!(input_port!(i64, "val"),)
	}
}

async fn example() -> BehaviorTreeResult {
	// create an external blackboard which will survive the tree
	let global_blackboard = Databoard::new();
	// BT-Trees blackboard has global blackboard as parent
	let root_blackboard = Databoard::with(Some(global_blackboard.clone()), None, false);

	let mut factory = BehaviorTreeFactory::with_groot2_behaviors()?;

	register_behavior!(factory, PrintNumber, "PrintNumber")?;

	factory.register_behavior_tree_from_text(XML)?;

	let mut tree = factory.create_tree_with("MainTree", root_blackboard)?;
	drop(factory);

	// direct interaction with the global blackboard
	for value in 1..=3 {
		global_blackboard.set("value", value)?;
		let result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);

		let value_sqr = global_blackboard.get::<i64>("@value_sqr")?;
		if value_sqr != value * value {
			return Ok(BehaviorState::Failure);
		}
		println!("[While loop] value: {value} value_sqr: {value_sqr}");

		let value_pow3 = global_blackboard.get::<i64>("@value_pow3")?;
		if value_pow3 != value * value * value {
			return Ok(BehaviorState::Failure);
		}

		let value_pow4 = global_blackboard.get::<i64>("@value_pow4")?;
		if value_pow4 != value * value * value * value {
			return Ok(BehaviorState::Failure);
		}
	}

	Ok(BehaviorState::Success)
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
	async fn t16_global_blackboard() -> Result<(), Error> {
		let result = example().await?;
		assert_eq!(result, BehaviorState::Success);
		Ok(())
	}
}
