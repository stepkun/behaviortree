// Copyright © 2025 Stephan Kunz
//! Implements the fourteenth tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev).
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-advanced/tutorial_14_subtree_model).
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t14_subtree_model.cpp).

mod common;

use behaviortree::prelude::*;
use common::test_data::SaySomething;

// region:		--- Example0
/// A completely manual remapping.
const XML0: &str = r#"
<root BTCPP_format="4">
  	<BehaviorTree ID="MainTree">
        <Sequence>
            <Script code="target:='1;2;3'"/>
            <SubTree ID="MoveRobot" target="{target}"  frame="world" result="{=}" />
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

async fn example0() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_core_behaviors()?;
	factory.register_test_behaviors()?;

	register_behavior!(factory, SaySomething, "SaySomething")?;
	// register subtrees behaviors
	move_robot::register_behaviors(&mut factory)?;

	factory.register_behavior_tree_from_text(XML0)?;

	let mut tree = factory.create_tree("MainTree")?;
	drop(factory);

	let result = tree.tick_while_running().await?;
	Ok(result)
}
// endregion:	--- Example0

// region:		--- Example1
/// A mix aof automatic & manual remapping.
const XML1: &str = r#"
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

async fn example1() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_core_behaviors()?;
	factory.register_test_behaviors()?;

	register_behavior!(factory, SaySomething, "SaySomething")?;
	// register subtrees behaviors
	move_robot::register_behaviors(&mut factory)?;

	factory.register_behavior_tree_from_text(XML1)?;

	let mut tree = factory.create_tree("MainTree")?;
	drop(factory);

	let result = tree.tick_while_running().await?;
	Ok(result)
}
// endregion:	--- Example1

// region:		--- Example2
/// using `TreeNodesModel` for remapping.
const XML2: &str = r#"
<root BTCPP_format="4">
  	<BehaviorTree ID="MainTree">
        <Sequence>
            <Script code="target:='1;2;3'"/>
            <SubTree ID="MoveRobot" />
            <SaySomething message="{error_code}"/>
        </Sequence>
  	</BehaviorTree>

	<TreeNodesModel>
		<SubTree ID="MoveRobot">
			<input_port  name="move_goal"  default="{target}"/>
			<input_port  name="frame"   default="world"/>
			<output_port name="result"  default="{error_code}"/>
		</SubTree>
  	</TreeNodesModel>

    <BehaviorTree ID="MoveRobot">
        <Fallback>
            <Sequence>
	            <SaySomething message="{frame}"/>
                <MoveBase goal="{move_goal}"/>
                <Script code="result:='goal_reached'"/>
            </Sequence>
            <ForceFailure>
                <Script code="result:='error'"/>
            </ForceFailure>
        </Fallback>
    </BehaviorTree>
</root>
"#;

async fn example2() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_core_behaviors()?;
	factory.register_test_behaviors()?;

	register_behavior!(factory, SaySomething, "SaySomething")?;
	// register subtrees behaviors
	move_robot::register_behaviors(&mut factory)?;

	factory.register_behavior_tree_from_text(XML2)?;

	let mut tree = factory.create_tree("MainTree")?;
	drop(factory);

	let result = tree.tick_while_running().await?;
	Ok(result)
}
// endregion:	--- Example2

// region:		--- Example3
/// The new example in BehaviorTree.CPP.
const XML3: &str = r#"
<root BTCPP_format="4">
  	<BehaviorTree ID="MainTree">
		<Sequence>
			<Script code="in_name:= 'john' "/>
			<SubTree ID="MySub" sub_in_name="{in_name}"
								sub_out_state="{out_state}"/>
			<ScriptCondition code=" out_result==69 &amp;&amp; out_state=='ACTIVE' " />
	        <SaySomething message="Success!"/>
		</Sequence>
  	</BehaviorTree>
</root>
"#;

const XML3_SUBTREE: &str = r#"
<root BTCPP_format="4">
  <TreeNodesModel>
    <SubTree ID="MySub">
      <input_port name="sub_in_value" default="42"/>
      <input_port name="sub_in_name"/>
      <output_port name="sub_out_result" default="{out_result}"/>
      <output_port name="sub_out_state"/>
    </SubTree>
  </TreeNodesModel>

  <BehaviorTree ID="MySub">
    <Sequence>
	  <!-- the original '&&' is a none valid xml, so it is replaced by '&amp;&amp;' -->
      <ScriptCondition code="sub_in_value==42 &amp;&amp; sub_in_name=='john'" />
      <Script code="sub_out_result:=69; sub_out_state:='ACTIVE'" />
    </Sequence>
  </BehaviorTree>
</root>
"#;

async fn example3() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_core_behaviors()?;
	register_behavior!(factory, SaySomething, "SaySomething")?;
	factory.register_behavior_tree_from_text(XML3_SUBTREE)?;
	factory.register_behavior_tree_from_text(XML3)?;

	let mut tree = factory.create_tree("MainTree")?;
	drop(factory);

	let result = tree.tick_while_running().await?;
	Ok(result)
}
// endregion:	--- Example3

#[tokio::main]
async fn main() -> Result<(), Error> {
	println!("running example 0");
	example0().await?;
	println!("running example 1");
	example1().await?;
	println!("running example 2");
	example2().await?;
	println!("running example 3");
	example3().await?;
	Ok(())
}

#[cfg(test)]
mod test {
	use super::*;

	#[tokio::test]
	async fn t14_subtree_model() -> Result<(), Error> {
		let result = example0().await?;
		assert_eq!(result, BehaviorState::Success);
		let result = example1().await?;
		assert_eq!(result, BehaviorState::Success);
		let result = example2().await?;
		assert_eq!(result, BehaviorState::Success);
		let result = example3().await?;
		assert_eq!(result, BehaviorState::Success);
		Ok(())
	}
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

			println!(
				"[ MoveBase: SEND REQUEST ]. goal: x={:2.1} y={:2.1} theta={:2.1}",
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
				println!("--- status: RUNNING");
				Ok(BehaviorState::Running)
			} else {
				println!("[ MoveBase: FINISHED ]");
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
