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
use embassy_futures::select::{Either3, select3};
use embedded_hal::digital::StatefulOutputPin;

// some constants
const TOGGLE_DELAY: u64 = 1500; // how long to wait on direct switching between the directions.

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
		let request = behavior.get::<String>("request")?;
		// info!("DoorMotorDriver: {}", request.as_str());
		let _request = behavior.set::<String>("command", request)?;
		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list! {input_port!(String, "request"), output_port!(String, "command")}
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
		let _command = behavior.get::<String>("command")?;
		// info!("Preparation for: {}", command.as_str());
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
		// info!("ReadControlButtons: tick()");
		if behavior.get::<bool>("stop_button")? {
			// info!("ReadControlButtons: Stop button is active");
			behavior.set("active_button", String::from("stop"))?
		} else if behavior.get::<bool>("up_button")? {
			// info!("ReadControlButtons: Up button is active");
			behavior.set("active_button", String::from("up"))?
		} else if behavior.get::<bool>("down_button")? {
			// info!("ReadControlButtons: Down button is active");
			behavior.set("active_button", String::from("down"))?
		} else {
			// info!("ReadControlButtons: No button is active");
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
		// info!("ReadEndContacts: tick()");
		let button = behavior.get::<String>("active_button")?;
		match button.as_ref() {
			"up" => {
				// info!("ReadEndContacts: up");
				if behavior.get::<bool>("upper_end")? {
					behavior.set("command", String::from("stop"))?;
				} else {
					behavior.set("command", String::from("up"))?;
				}
			}
			"down" => {
				// info!("ReadEndContacts: down");
				if behavior.get::<bool>("lower_end")? {
					behavior.set("command", String::from("stop"))?;
				} else {
					behavior.set("command", String::from("down"))?;
				}
			}
			_ => {
				// info!("ReadEndContacts: _");
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

	let mut result = tree.tick_once().await?;
	while result == BehaviorState::Running {
		Timer::after_millis(100).await;
		result = tree.tick_once().await?;
	}
	Ok(result)
}

#[ariel_os::task(autostart, peripherals)]
#[allow(unsafe_code)]
async fn handle_motor(peripherals: pins::MotorPeripherals) {
	Timer::after_millis(30).await;
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
	info!("   initializing motor");
	let mut motor_up = Output::new(peripherals.motor_up, Level::Low);
	let mut motor_down = Output::new(peripherals.motor_down, Level::Low);

	loop {
		let command = blackboard
			.get::<String>("command")
			.unwrap_or_else(|_| alloc::string::String::from("stop"));
		match command.as_str() {
			"stop" => {
				// info!("motor stop");
				motor_up.set_low();
				motor_down.set_low();
			}
			"up" => {
				// info!("motor up");
				if motor_down.is_set_high().expect("snh") {
					motor_down.set_low();
					Timer::after_millis(TOGGLE_DELAY).await;
				}
				motor_up.set_high();
			}
			"down" => {
				// info!("motor down");
				if motor_up.is_set_high().expect("snh") {
					motor_up.set_low();
					Timer::after_millis(TOGGLE_DELAY).await;
				}
				motor_down.set_high();
			}
			_ => {
				info!("motor weird command: {}", command.as_str());
				motor_up.set_low();
				motor_down.set_low();
			}
		};
		Timer::after_millis(10).await;
	}
}

#[ariel_os::task(autostart, peripherals)]
#[allow(unsafe_code)]
async fn handle_panel(peripherals: pins::PanelPeripherals) {
	Timer::after_millis(20).await;
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

	info!("   initializing panel");
	let mut led_up = Output::new(peripherals.led_up, Level::Low);
	let mut led_down = Output::new(peripherals.led_down, Level::Low);

	let pull = Pull::Down;
	let mut btn_up = Input::builder(peripherals.btn_up, pull)
		.build_with_interrupt()
		.unwrap();
	let mut btn_stop = Input::builder(peripherals.btn_stop, pull)
		.build_with_interrupt()
		.unwrap();
	let mut btn_down = Input::builder(peripherals.btn_down, pull)
		.build_with_interrupt()
		.unwrap();

	info!("   running panel loop");
	loop {
		// Wait for panel button being pressed.
		let button = select3(btn_stop.wait_for_high(), btn_up.wait_for_high(), btn_down.wait_for_high()).await;
		match button {
			Either3::First(_) => {
				// info!("stop pressed");
				blackboard.set("stop_button", true).expect("snh");
				blackboard.set("up_button", false).expect("snh");
				led_up.set_low();
				blackboard.set("down_button", false).expect("snh");
				led_down.set_low();
			}
			Either3::Second(_) => {
				// info!("up pressed");
				blackboard.set("stop_button", false).expect("snh");
				blackboard.set("down_button", false).expect("snh");
				led_down.set_low();
				blackboard.set("up_button", true).expect("snh");
				led_up.set_high();
			}
			Either3::Third(_) => {
				// info!("down pressed");
				blackboard.set("stop_button", false).expect("snh");
				blackboard.set("up_button", false).expect("snh");
				led_up.set_low();
				blackboard.set("down_button", true).expect("snh");
				led_down.set_high();
			}
		}
	}
}

#[ariel_os::task(autostart, peripherals)]
#[allow(unsafe_code)]
async fn handle_security(peripherals: pins::SecurityPeripherals) {
	Timer::after_millis(10).await;
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

	info!("   initializing security");
	let mut led_upper_end = Output::new(peripherals.led_upper_end, Level::Low);
	let mut led_lower_end = Output::new(peripherals.led_lower_end, Level::Low);

	let pull = Pull::Down;
	let mut btn_upper_end = Input::builder(peripherals.btn_upper_end, pull)
		.build_with_interrupt()
		.unwrap();
	let mut btn_lower_end = Input::builder(peripherals.btn_lower_end, pull)
		.build_with_interrupt()
		.unwrap();

	info!("   running security loop");
	loop {
		// Wait for end contacts being touched.
		let button = select3(
			btn_upper_end.wait_for_high(),
			btn_lower_end.wait_for_high(),
			Timer::after_millis(100),
		)
		.await;
		match button {
			Either3::First(_) => {
				// info!("upper end");
				blackboard.set("upper_end", true).expect("snh");
				led_upper_end.set_high();
				blackboard.set("lower_end", false).expect("snh");
				led_lower_end.set_low();
			}
			Either3::Second(_) => {
				// info!("lower end");
				blackboard.set("lower_end", true).expect("snh");
				led_lower_end.set_high();
				blackboard.set("upper_end", false).expect("snh");
				led_upper_end.set_low();
			}
			Either3::Third(_) => {
				// info!("tineout");
				blackboard.set("lower_end", false).expect("snh");
				led_lower_end.set_low();
				blackboard.set("upper_end", false).expect("snh");
				led_upper_end.set_low();
			}
		}
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
