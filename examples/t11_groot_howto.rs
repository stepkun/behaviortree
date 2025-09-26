// Copyright © 2025 Stephan Kunz
//! Implements the eleventh tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev).
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-basics/tutorial_11_groot2).
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t11_groot_howto.cpp).

mod common;

use behaviortree::{Groot2Connector, XmlCreator, prelude::*};
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
#[derive(Action, Debug, Default)]
pub struct UpdatePosition {
	pos: Position2D,
}

#[async_trait::async_trait]
impl Behavior for UpdatePosition {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		self.pos.x += 0.2;
		self.pos.y += 0.1;
		behavior.set("pos", self.pos.clone())?;
		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list![output_port!(Position2D, "pos")]
	}
}

// @TODO: groot publishing missing
async fn example() -> Result<(BehaviorState, BehaviorTree), Error> {
	let mut factory = BehaviorTreeFactory::with_groot2_behaviors()?;

	// special creation/registration of multiple methods of a struct
	#[cfg(test)]
	let _crossdoor = CrossDoor::register_behaviors(&mut factory)?;

	#[cfg(not(test))]
	let crossdoor = CrossDoor::register_behaviors(&mut factory)?;
	// Behavior registration, as usual
	register_behavior!(factory, UpdatePosition, "UpdatePosition")?;

	// Groot2 editor requires a model of your registered behaviors.
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
	println!("----------- XML file  ----------");
	println!("{}", &xml);
	println!("--------------------------------");

	// Connect the Groot2Publisher. This will allow Groot2 to
	// get the tree and poll status updates.
	let _publisher = Groot2Connector::new(&mut tree, 5555);

	#[cfg(test)]
	let result = tree.tick_while_running().await?;

	#[cfg(not(test))]
	loop {
		println!("Start");
		tree.reset()?;
		crossdoor.lock().reset();
		tree.tick_while_running().await?;

		tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
	}

	#[cfg(test)]
	Ok((result, tree))
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
	async fn t11_groot_howto() -> Result<(), Error> {
		let result = example().await?;
		assert_eq!(result.0, BehaviorState::Success);

		let metadata_xml = XmlCreator::write_tree(&result.1, true, false, true)?;
		assert_eq!(METADATA_RESULT, metadata_xml.as_ref());

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
    <Action ID="SmashDoor"/>
    <Action ID="UpdatePosition">
      <output_port name="pos" type="Position2D"/>
    </Action>
  </TreeNodesModel>
</root>"#;

#[allow(unused)]
const METADATA_RESULT: &str = r#"<root BTCPP_format="4">
  <BehaviorTree ID="MainTree" _fullpath="MainTree::0">
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
    <Action ID="SmashDoor"/>
    <Action ID="UpdatePosition">
      <output_port name="pos" type="Position2D"/>
    </Action>
  </TreeNodesModel>
</root>"#;
