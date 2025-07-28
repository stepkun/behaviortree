// Copyright © 2025 Stephan Kunz

//! This test implements the eleventh tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev)
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-basics/tutorial_11_groot2)
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t11_groot_howto.cpp)
//!

extern crate alloc;
mod common;

use behaviortree::{
    Behavior, Groot2Connector, SharedRuntime, XmlCreator,
    behavior::{
        BehaviorData, BehaviorInstance, BehaviorKind, BehaviorResult, BehaviorState, BehaviorStatic,
    },
    factory::BehaviorTreeFactory,
    output_port,
    port::PortList,
    port_list, register_behavior,
    tree::{BehaviorTree, ConstBehaviorTreeElementList},
};
use common::cross_door::CrossDoor;
use common::test_data::Position2D;

const XML: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="MainTree">
		<Sequence>
			<Script code="door_open:=false" />
			<UpdatePosition pos="{pos_2D}" />
			<Fallback>
				<Inverter>
					<IsDoorClosed/>
				</Inverter>
				<SubTree ID="DoorClosed" _autoremap="true" door_open="{door_open}"/>
			</Fallback>
			<PassThroughDoor/>
		</Sequence>
	</BehaviorTree>

	<BehaviorTree ID="DoorClosed">
		<Fallback name="tryOpen" _onSuccess="door_open:=true">
			<OpenDoor/>
			<RetryUntilSuccessful num_attempts="5">
				<PickLock/>
			</RetryUntilSuccessful>
			<SmashDoor/>
		</Fallback>
	</BehaviorTree>
</root>
"#;

/// Action `UpdatePosition`
#[derive(Behavior, Debug, Default)]
pub struct UpdatePosition {
    pos: Position2D,
}

#[async_trait::async_trait]
impl BehaviorInstance for UpdatePosition {
    async fn tick(
        &mut self,
        behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> BehaviorResult {
        self.pos.x += 0.2;
        self.pos.y += 0.1;
        behavior.set("pos", self.pos.clone())?;
        Ok(BehaviorState::Success)
    }
}

impl BehaviorStatic for UpdatePosition {
    fn kind() -> BehaviorKind {
        BehaviorKind::Action
    }

    fn provided_ports() -> PortList {
        port_list![output_port!(Position2D, "pos")]
    }
}

// @TODO: groot publishing missing
async fn example() -> anyhow::Result<(BehaviorState, BehaviorTree)> {
    let mut factory = BehaviorTreeFactory::with_groot2_behaviors()?;

    // special creation/registration uf multiple methods of a struct
    let crossdoor = CrossDoor::register_behaviors(&mut factory)?;
    // Nodes registration, as usual
    register_behavior!(factory, UpdatePosition, "UpdatePosition")?;

    // Groot2 editor requires a model of your registered Nodes.
    // You don't need to write that by hand, it can be automatically
    // generated using the following command.
    let xml_model = XmlCreator::write_tree_nodes_model(&factory, true)?;
    println!("-------- TreeNodesModel --------");
    println!("{xml_model}");
    println!("--------------------------------");

    factory.register_behavior_tree_from_text(XML)?;

    // Add this to allow Groot2 to visualize your custom type
    // @TODO:
    //BT::RegisterJsonDefinition<Position2D>();

    let mut tree = factory.create_tree("MainTree")?;
    drop(factory);

    // Print the full tree with model
    let xml = XmlCreator::write_tree(&tree, false, false, true)?;
    assert_eq!(RESULT, xml.as_ref());
    // let xml = XmlCreator::groot_write_tree(&tree)?;
    println!("----------- XML file  ----------");
    println!("{}", &xml);
    println!("--------------------------------");

    // Connect the Groot2Publisher. This will allow Groot2 to
    // get the tree and poll status updates.
    let _publisher = Groot2Connector::new(&mut tree, 1667);

    #[cfg(test)]
    let result = tree.tick_while_running().await?;

    #[cfg(not(test))]
    loop {
        println!("Start");
        tree.reset().await?;
        crossdoor.lock().reset();
        // tree.tick_exactly_once().await?.is_completed() {
        // 	tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        // }
        tree.tick_while_running().await?;

        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    }

    #[cfg(test)]
    Ok((result, tree))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    example().await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn t11_groot_howto() -> anyhow::Result<()> {
        let result = example().await?;
        assert_eq!(result.0, BehaviorState::Success);

        let metadata_xml = XmlCreator::write_tree(&result.1, true, false, true)?;
        assert_eq!(METADATA_RESULT, metadata_xml.as_ref());

        // let full_xml = XmlCreator::write_tree(&tree, true, true, true)?;
        // assert_eq!(FULL_RESULT, full_xml.as_ref());

        Ok(())
    }
}

#[allow(unused)]
const RESULT: &str = r#"<root BTCPP_format="4">
  <BehaviorTree ID="MainTree" _fullpath="">
    <Sequence name="Sequence">
      <Script name="Script" code="door_open:=false"/>
      <UpdatePosition name="UpdatePosition" pos="{pos_2D}"/>
      <Fallback name="Fallback">
        <Inverter name="Inverter">
          <IsDoorClosed name="IsDoorClosed"/>
        </Inverter>
        <SubTree ID="DoorClosed" door_open="{door_open}"/>
      </Fallback>
      <PassThroughDoor name="PassThroughDoor"/>
    </Sequence>
  </BehaviorTree>
  <BehaviorTree ID="DoorClosed" _fullpath="DoorClosed::7">
    <Fallback name="tryOpen" _onSuccess="door_open:=true">
      <OpenDoor name="OpenDoor"/>
      <RetryUntilSuccessful name="RetryUntilSuccessful" num_attempts="5">
        <PickLock name="PickLock"/>
      </RetryUntilSuccessful>
      <SmashDoor name="SmashDoor"/>
    </Fallback>
  </BehaviorTree>
  <TreeNodesModel>
    <Condition ID="IsDoorClosed"/>
    <Action ID="OpenDoor"/>
    <Action ID="PassThroughDoor"/>
    <Action ID="PickLock"/>
    <Condition ID="SmashDoor"/>
    <Action ID="UpdatePosition">
      <output_port name="pos" type="Position2D"/>
    </Action>
  </TreeNodesModel>
</root>"#;

#[allow(unused)]
const METADATA_RESULT: &str = r#"<root BTCPP_format="4">
  <BehaviorTree ID="MainTree" _fullpath="">
    <Sequence name="Sequence" _uid="1">
      <Script name="Script" _uid="2" code="door_open:=false"/>
      <UpdatePosition name="UpdatePosition" _uid="3" pos="{pos_2D}"/>
      <Fallback name="Fallback" _uid="4">
        <Inverter name="Inverter" _uid="5">
          <IsDoorClosed name="IsDoorClosed" _uid="6"/>
        </Inverter>
        <SubTree ID="DoorClosed" _fullpath="DoorClosed::7" _uid="7" door_open="{door_open}"/>
      </Fallback>
      <PassThroughDoor name="PassThroughDoor" _uid="13"/>
    </Sequence>
  </BehaviorTree>
  <BehaviorTree ID="DoorClosed" _fullpath="DoorClosed::7">
      <Fallback name="tryOpen" _uid="8" _onSuccess="door_open:=true">
        <OpenDoor name="OpenDoor" _uid="9"/>
        <RetryUntilSuccessful name="RetryUntilSuccessful" _uid="10" num_attempts="5">
          <PickLock name="PickLock" _uid="11"/>
        </RetryUntilSuccessful>
        <SmashDoor name="SmashDoor" _uid="12"/>
      </Fallback>
  </BehaviorTree>
  <TreeNodesModel>
    <Condition ID="IsDoorClosed"/>
    <Action ID="OpenDoor"/>
    <Action ID="PassThroughDoor"/>
    <Action ID="PickLock"/>
    <Condition ID="SmashDoor"/>
    <Action ID="UpdatePosition">
      <output_port name="pos" type="Position2D"/>
    </Action>
  </TreeNodesModel>
</root>"#;

#[allow(unused)]
const FULL_RESULT: &str = r#"<root BTCPP_format="4">
  <BehaviorTree ID="MainTree" _fullpath="">
    <Sequence name="Sequence" _uid="1">
      <Script name="Script" _uid="2" code="door_open:=false"/>
      <UpdatePosition name="UpdatePosition" _uid="3" pos="{pos_2D}"/>
      <Fallback name="Fallback" _uid="4">
        <Inverter name="Inverter" _uid="5">
          <IsDoorClosed name="IsDoorClosed" _uid="6"/>
        </Inverter>
        <SubTree ID="DoorClosed" _fullpath="DoorClosed::7" _uid="7" door_open="{door_open}"/>
      </Fallback>
      <PassThroughDoor name="PassThroughDoor" _uid="13"/>
    </Sequence>
  </BehaviorTree>
  <BehaviorTree ID="DoorClosed" _fullpath="DoorClosed::7">
    <Fallback name="tryOpen" _uid="8" _onSuccess="door_open:=true">
      <OpenDoor name="OpenDoor" _uid="9"/>
      <RetryUntilSuccessful name="RetryUntilSuccessful" _uid="10" num_attempts="5">
        <PickLock name="PickLock" _uid="11"/>
      </RetryUntilSuccessful>
      <SmashDoor name="SmashDoor" _uid="12"/>
    </Fallback>
  </BehaviorTree>
  <TreeNodesModel>
    <Action ID="AlwaysFailure"/>
    <Action ID="AlwaysSuccess"/>
    <Control ID="AsyncFallback"/>
    <Control ID="AsyncSequence"/>
    <Decorator ID="Delay">
      <input_port name="delay_msec" type="unsigned int">Tick the child after a few milliseconds</input_port>
    </Decorator>
    <Control ID="Fallback"/>
    <Decorator ID="ForceFailure"/>
    <Decorator ID="ForceSuccess"/>
    <Control ID="IfThenElse"/>
    <Decorator ID="Inverter"/>
    <Condition ID="IsDoorClosed"/>
    <Decorator ID="KeepRunningUntilFailure"/>
    <Decorator ID="LoopBool">
      <output_port name="value" type="bool"/>
      <input_port name="if_empty" type="BT::NodeStatus" default="SUCCESS">Status to return if queue is empty: SUCCESS, FAILURE, SKIPPED</input_port>
      <inout_port name="queue" type="std::shared_ptr&lt;std::deque&lt;bool, std::allocator&lt;bool&gt; &gt; &gt;"/>
    </Decorator>
    <Decorator ID="LoopDouble">
      <output_port name="value" type="double"/>
      <input_port name="if_empty" type="BT::NodeStatus" default="SUCCESS">Status to return if queue is empty: SUCCESS, FAILURE, SKIPPED</input_port>
      <inout_port name="queue" type="std::shared_ptr&lt;std::deque&lt;double, std::allocator&lt;double&gt; &gt; &gt;"/>
    </Decorator>
    <Decorator ID="LoopInt">
      <output_port name="value" type="int"/>
      <input_port name="if_empty" type="BT::NodeStatus" default="SUCCESS">Status to return if queue is empty: SUCCESS, FAILURE, SKIPPED</input_port>
      <inout_port name="queue" type="std::shared_ptr&lt;std::deque&lt;int, std::allocator&lt;int&gt; &gt; &gt;"/>
    </Decorator>
    <Decorator ID="LoopString">
      <output_port name="value" type="std::string"/>
      <input_port name="if_empty" type="BT::NodeStatus" default="SUCCESS">Status to return if queue is empty: SUCCESS, FAILURE, SKIPPED</input_port>
      <inout_port name="queue" type="std::shared_ptr&lt;std::deque&lt;std::__cxx11::basic_string&lt;char, std::char_traits&lt;char&gt;, std::allocator&lt;char&gt; &gt;, std::allocator&lt;std::__cxx11::basic_string&lt;char, std::char_traits&lt;char&gt;, std::allocator&lt;char&gt; &gt; &gt; &gt; &gt;"/>
    </Decorator>
    <Action ID="OpenDoor"/>
    <Control ID="Parallel">
      <input_port name="failure_count" type="int" default="1">number of children that need to fail to trigger a FAILURE</input_port>
      <input_port name="success_count" type="int" default="-1">number of children that need to succeed to trigger a SUCCESS</input_port>
    </Control>
    <Control ID="ParallelAll">
      <input_port name="max_failures" type="int" default="1">If the number of children returning FAILURE exceeds this value, ParallelAll returns FAILURE</input_port>
    </Control>
    <Action ID="PassThroughDoor"/>
    <Action ID="PickLock"/>
    <Decorator ID="Precondition">
      <input_port name="else" type="BT::NodeStatus" default="FAILURE">Return status if condition is false</input_port>
      <input_port name="if" type="std::string"/>
    </Decorator>
    <Control ID="ReactiveFallback"/>
    <Control ID="ReactiveSequence"/>
    <Decorator ID="Repeat">
      <input_port name="num_cycles" type="int">Repeat a successful child up to N times. Use -1 to create an infinite loop.</input_port>
    </Decorator>
    <Decorator ID="RetryUntilSuccessful">
      <input_port name="num_attempts" type="int">Execute again a failing child up to N times. Use -1 to create an infinite loop.</input_port>
    </Decorator>
    <Decorator ID="RunOnce">
      <input_port name="then_skip" type="bool" default="true">If true, skip after the first execution, otherwise return the same NodeStatus returned once by the child.</input_port>
    </Decorator>
    <Action ID="Script">
      <input_port name="code" type="std::string">Piece of code that can be parsed</input_port>
    </Action>
    <Condition ID="ScriptCondition">
      <input_port name="code" type="BT::AnyTypeAllowed">Piece of code that can be parsed. Must return false or true</input_port>
    </Condition>
    <Control ID="Sequence"/>
    <Control ID="SequenceWithMemory"/>
    <Action ID="SetBlackboard">
      <inout_port name="output_key" type="BT::AnyTypeAllowed">Name of the blackboard entry where the value should be written</inout_port>
      <input_port name="value" type="BT::AnyTypeAllowed">Value to be written into the output_key</input_port>
    </Action>
    <Decorator ID="SkipUnlessUpdated">
      <input_port name="entry" type="BT::Any">Entry to check</input_port>
    </Decorator>
    <Action ID="Sleep">
      <input_port name="msec" type="unsigned int"/>
    </Action>
    <Condition ID="SmashDoor"/>
    <SubTree ID="SubTree">
      <input_port name="_autoremap" type="bool" default="false">If true, all the ports with the same name will be remapped</input_port>
    </SubTree>
    <Control ID="Switch2">
      <input_port name="case_2" type="std::string"/>
      <input_port name="case_1" type="std::string"/>
      <input_port name="variable" type="std::string"/>
    </Control>
    <Control ID="Switch3">
      <input_port name="case_3" type="std::string"/>
      <input_port name="case_2" type="std::string"/>
      <input_port name="case_1" type="std::string"/>
      <input_port name="variable" type="std::string"/>
    </Control>
    <Control ID="Switch4">
      <input_port name="case_4" type="std::string"/>
      <input_port name="case_3" type="std::string"/>
      <input_port name="case_2" type="std::string"/>
      <input_port name="case_1" type="std::string"/>
      <input_port name="variable" type="std::string"/>
    </Control>
    <Control ID="Switch5">
      <input_port name="case_5" type="std::string"/>
      <input_port name="case_4" type="std::string"/>
      <input_port name="case_3" type="std::string"/>
      <input_port name="case_2" type="std::string"/>
      <input_port name="case_1" type="std::string"/>
      <input_port name="variable" type="std::string"/>
    </Control>
    <Control ID="Switch6">
      <input_port name="case_5" type="std::string"/>
      <input_port name="case_4" type="std::string"/>
      <input_port name="case_6" type="std::string"/>
      <input_port name="case_3" type="std::string"/>
      <input_port name="case_2" type="std::string"/>
      <input_port name="case_1" type="std::string"/>
      <input_port name="variable" type="std::string"/>
    </Control>
    <Decorator ID="Timeout">
      <input_port name="msec" type="unsigned int">After a certain amount of time, halt() the child if it is still running.</input_port>
    </Decorator>
    <Action ID="UnsetBlackboard">
      <input_port name="key" type="std::string">Key of the entry to remove</input_port>
    </Action>
    <Action ID="UpdatePosition">
      <output_port name="pos" type="Position2D"/>
    </Action>
    <Decorator ID="WaitValueUpdate">
      <input_port name="entry" type="BT::Any">Entry to check</input_port>
    </Decorator>
    <Action ID="WasEntryUpdated">
      <input_port name="entry" type="BT::Any">Entry to check</input_port>
    </Action>
    <Control ID="WhileDoElse"/>
  </TreeNodesModel>
</root>"#;
