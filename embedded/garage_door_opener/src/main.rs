// Copyright © 2025 Stephan Kunz
//! A garage door opener implementation.

#![no_main]
#![no_std]
#![allow(static_mut_refs)]

mod pins;

use ariel_os::{
	debug::{ExitCode, exit, log::*},
	gpio::{Input, Level, Output, Pull},
	time::Timer,
};
use behaviortree::prelude::*;

// include the Groot2 behavior file
const XML: &str = include_str!("GarageDoorOpener.xml");

// a truly global blackboard
static mut GLOBAL_BLACKBOARD: Option<Databoard> = None;

#[derive(Action, Debug, Default)]
struct DoorMotorDriver {}

#[async_trait::async_trait]
impl Behavior for DoorMotorDriver {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let command = behavior.get::<String>("command")?;
		info!("DoorMotorDriver: {}", command.as_str());
		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list! {input_port!(String, "command")}
	}
}

#[derive(Condition, Debug, Default)]
struct EmergencyOffActive {}

#[async_trait::async_trait]
impl Behavior for EmergencyOffActive {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let emergency = behavior.get::<bool>("emergency")?;
		if emergency {
			info!("Emergency Active");
			Ok(BehaviorState::Success)
		} else {
			Ok(BehaviorState::Failure)
		}
	}

	fn provided_ports() -> PortList {
		port_list! {input_port!(bool, "emergency")}
	}
}

#[derive(Action, Debug, Default)]
struct Preparation {}

#[async_trait::async_trait]
impl Behavior for Preparation {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		let command = behavior.get::<String>("command")?;
		info!("Preparation for: {}", command.as_str());
		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list! {input_port!(String, "command")}
	}
}

#[derive(Action, Debug, Default)]
struct ReadControlButtons {}

#[async_trait::async_trait]
impl Behavior for ReadControlButtons {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		info!("ReadControlButtons: tick()");
		if behavior.get::<bool>("stop_button")? {
			behavior.set("active_button", String::from("stop"))?
		} else if behavior.get::<bool>("up_button")? {
			behavior.set("active_button", String::from("up"))?
		} else if behavior.get::<bool>("down_button")? {
			behavior.set("active_button", String::from("down"))?
		} else {
			behavior.set("active_button", String::from("stop"))?
		};
		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list! {
			input_port!(bool, "stop_button", true),
			input_port!(bool, "up_button", false),
			input_port!(bool, "down_button", false),
			output_port!(String, "active_button", "stop"),
		}
	}
}

#[derive(Action, Debug, Default)]
struct ReadEndContacts {}

#[async_trait::async_trait]
impl Behavior for ReadEndContacts {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		info!("ReadControlButtons: tick()");
		let button = behavior.get::<String>("active_button")?;
		match button.as_ref() {
			"up" => {
				if !behavior.get::<bool>("upper_end")? {
					behavior.set("command", String::from("up"))?;
				} else {
					behavior.set("command", String::from("stop"))?;
				}
			}
			"down" => {
				if !behavior.get::<bool>("lower_end")? {
					behavior.set("command", String::from("down"))?;
				} else {
					behavior.set("command", String::from("stop"))?;
				}
			}
			_ => {
				behavior.set("command", String::from("stop"))?;
			}
		}
		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list! {
			input_port!(String, "active_button", "stop"),
			input_port!(bool, "lower_end", true),
			input_port!(bool, "upper_end", true),
			output_port!(String, "command", "stop"),
		}
	}
}

#[allow(unsafe_code)]
async fn behavior() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_extended_behaviors()?;

	register_behavior!(factory, DoorMotorDriver, "DoorMotorDriver")?;
	register_behavior!(factory, EmergencyOffActive, "EmergencyOffActive")?;
	register_behavior!(factory, Preparation, "Preparation")?;
	register_behavior!(factory, ReadControlButtons, "ReadControlButtons")?;
	register_behavior!(factory, ReadEndContacts, "ReadEndContacts")?;
	factory.register_behavior_tree_from_text(XML)?;

	// @TODO: replace with lazy lock
	let blackboard = unsafe {
		if let Some(blackboard) = &GLOBAL_BLACKBOARD {
			blackboard.clone()
		} else {
			let blackboard = Databoard::new();
			GLOBAL_BLACKBOARD = Some(blackboard.clone());
			blackboard
		}
	};
	// pre set blackboard variables
	blackboard.set::<bool>("emergency", false)?;
	// blackboard.set("active_button", String::from("stop"))?;
	// blackboard.set("command", String::from("stop"))?;
	// blackboard.set("motor_command", String::from("stop"))?;
	blackboard.set::<bool>("lower_end", true)?;
	blackboard.set::<bool>("upper_end", true)?;
	blackboard.set::<bool>("stop_button", true)?;
	blackboard.set::<bool>("up_button", false)?;
	blackboard.set::<bool>("down_button", false)?;
	// create the tree with the global blackboard
	let mut tree = factory.create_tree_with("GarageDoorOpener", &blackboard)?;

	// dropping the factory to free memory
	drop(factory);

	let result = tree.tick_while_running().await?;
	Ok(result)
}

#[ariel_os::task(autostart, peripherals)]
#[allow(unsafe_code)]
async fn handle_hardware(peripherals: pins::Peripherals) {
	// @TODO: replace with lazy lock
	let _blackboard = unsafe {
		if let Some(blackboard) = &GLOBAL_BLACKBOARD {
			blackboard.clone()
		} else {
			let blackboard = Databoard::new();
			GLOBAL_BLACKBOARD = Some(blackboard.clone());
			blackboard
		}
	};
	info!("   initializing hardware");
	let mut led_up = Output::new(peripherals.led_up, Level::Low);

	let pull = Pull::Down;

	let mut btn_up = Input::builder(peripherals.btn_up, pull)
		.build_with_interrupt()
		.unwrap();

	info!("   running hardware loop");
	loop {
		// Wait for the button being pressed or 300 ms, whichever comes first.
		let _ = embassy_futures::select::select(btn_up.wait_for_low(), Timer::after_millis(300)).await;

		led_up.toggle();
		Timer::after_millis(100).await;
	}
}

#[ariel_os::task(autostart)]
async fn main() {
	info!("running garage_door_opener...");
	match behavior().await {
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
