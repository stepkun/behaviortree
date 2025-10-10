// Copyright © 2025 Stephan Kunz
//! A garage door opener implementation.

#![no_main]
#![no_std]
#![allow(static_mut_refs)]

mod pins;

use ariel_os::{
	debug::{ExitCode, exit, log::*},
	gpio::{Input, Level, Output, Pull},
	time::{Duration, Instant, Timer},
};
use behaviortree::prelude::*;
use embassy_futures::select::{Either, Either3, select, select3};
use embedded_hal::digital::StatefulOutputPin;

// some configuration constants
const TOGGLE_DELAY: u64 = 500; // how long to wait in millisecs on direct switching between the directions.
const PREPARATION_TIME: u64 = 1500; // preparation timer value

// include the Groot2 behavior file
const XML: &str = include_str!("GarageDoorOpener.xml");

// a truly global blackboard
static mut GLOBAL_BLACKBOARD: Option<Databoard> = None;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
enum MotorCommand {
	Down,
	#[default]
	Stop,
	Up,
}

impl FromStr for MotorCommand {
	type Err = core::convert::Infallible;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		// info!("Converting string: \"{}\"", value.as_str());
		match value {
			"stop" => Ok(MotorCommand::Stop),
			"down" => Ok(MotorCommand::Down),
			"up" => Ok(MotorCommand::Up),
			_ => {
				info!("weird motor command");
				Ok(MotorCommand::Stop)
			}
		}
	}
}

impl core::fmt::Display for MotorCommand {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			MotorCommand::Stop => write!(f, "stop"),
			MotorCommand::Down => write!(f, "down"),
			MotorCommand::Up => write!(f, "up"),
		}
	}
}

#[derive(Action, Default)]
struct DoorMotorDriver;

#[async_trait::async_trait]
impl Behavior for DoorMotorDriver {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		if behavior.get::<bool>("emergency")? {
			behavior.set::<MotorCommand>("command", MotorCommand::Stop)?;
		} else {
			let request = behavior.get::<MotorCommand>("request")?;
			// info!("DoorMotorDriver: {}", request.as_str());
			behavior.set::<MotorCommand>("command", request)?;
		}
		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list! {input_port!(bool, "emergency"), input_port!(MotorCommand, "request"), output_port!(MotorCommand, "command")}
	}
}

#[derive(Condition, Default)]
struct EmergencyOffActive;

#[async_trait::async_trait]
impl Behavior for EmergencyOffActive {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		if behavior.get::<bool>("emergency")? {
			// info!("Emergency Active");
			Ok(BehaviorState::Success)
		} else {
			Ok(BehaviorState::Failure)
		}
	}

	fn provided_ports() -> PortList {
		port_list! {input_port!(bool, "emergency")}
	}
}

#[derive(Action, Default)]
struct Preparation {
	last_command: MotorCommand,
	start: Option<Instant>,
}

#[async_trait::async_trait]
impl Behavior for Preparation {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		// info!("Preparation tick()");
		let command = behavior.get::<MotorCommand>("command")?;
		// info!("command: {}", command.to_string().as_str());
		if command != self.last_command {
			self.last_command = command;
			if command != MotorCommand::Stop {
				behavior.set("preparation", true)?;
				self.start = Some(Instant::now());
				info!("timer started");
			} else {
				behavior.set("preparation", false)?;
				if self.start.is_some() {
					self.start = None;
					info!("timer canceled");
				}
			}
		} else if let Some(start) = self.start {
			if Instant::now().duration_since(start) >= Duration::from_millis(PREPARATION_TIME) {
				self.start = None;
				behavior.set("preparation", false)?;
				info!("timer finished");
			}
		}

		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list! {input_port!(MotorCommand, "command"), output_port!(bool, "preparation")}
	}
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum ControlButton {
	Down,
	Stop,
	Up,
}

impl FromStr for ControlButton {
	type Err = core::convert::Infallible;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		// info!("Converting string: \"{}\"", value.as_str());
		match value {
			"stop" => Ok(ControlButton::Stop),
			"down" => Ok(ControlButton::Down),
			"up" => Ok(ControlButton::Up),
			_ => {
				info!("weird button");
				Ok(ControlButton::Stop)
			}
		}
	}
}

impl core::fmt::Display for ControlButton {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			ControlButton::Stop => write!(f, "stop"),
			ControlButton::Down => write!(f, "down"),
			ControlButton::Up => write!(f, "up"),
		}
	}
}

#[derive(Action, Default)]
struct ReadControlButtons;

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
			behavior.set("active_button", ControlButton::Stop)?
		} else if behavior.get::<bool>("up_button")? {
			// info!("ReadControlButtons: Up button is active");
			behavior.set("active_button", ControlButton::Up)?
		} else if behavior.get::<bool>("down_button")? {
			// info!("ReadControlButtons: Down button is active");
			behavior.set("active_button", ControlButton::Down)?
		} else {
			// info!("ReadControlButtons: No button is active");
			behavior.set("active_button", ControlButton::Stop)?
		};
		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list! {
			input_port!(bool, "stop_button", true),
			input_port!(bool, "up_button", false),
			input_port!(bool, "down_button", false),
			output_port!(ControlButton, "active_button", ControlButton::Stop),
		}
	}
}

#[derive(Action, Default)]
struct ReadEndContacts;

#[async_trait::async_trait]
impl Behavior for ReadEndContacts {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		// info!("ReadEndContacts: tick()");
		let button = behavior.get::<ControlButton>("active_button")?;
		match button {
			ControlButton::Stop => {
				// info!("ReadEndContacts: _");
				behavior.set("command", MotorCommand::Stop)?;
			}
			ControlButton::Up => {
				// info!("ReadEndContacts: up");
				if behavior.get::<bool>("upper_end")? {
					behavior.set("command", MotorCommand::Stop)?;
				} else {
					behavior.set("command", MotorCommand::Up)?;
				}
			}
			ControlButton::Down => {
				// info!("ReadEndContacts: down");
				if behavior.get::<bool>("lower_end")? {
					behavior.set("command", MotorCommand::Stop)?;
				} else {
					behavior.set("command", MotorCommand::Down)?;
				}
			}
		}
		Ok(BehaviorState::Success)
	}

	fn provided_ports() -> PortList {
		port_list! {
			input_port!(ControlButton, "active_button", ControlButton::Stop),
			input_port!(bool, "lower_end", true),
			input_port!(bool, "upper_end", true),
			output_port!(MotorCommand, "command", MotorCommand::Stop),
		}
	}
}

#[allow(unsafe_code)]
async fn behavior() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::new()?;

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
	blackboard.set::<bool>("lower_end", true)?;
	blackboard.set::<bool>("upper_end", true)?;
	blackboard.set::<bool>("stop_button", true)?;
	blackboard.set::<bool>("up_button", false)?;
	blackboard.set::<bool>("down_button", false)?;
	blackboard.set::<bool>("preparation", false)?;
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
			.get::<MotorCommand>("command")
			.unwrap_or_else(|_| MotorCommand::Stop);
		match command {
			MotorCommand::Stop => {
				// info!("motor stop");
				motor_up.set_low();
				motor_down.set_low();
			}
			MotorCommand::Up => {
				// info!("motor up");
				if motor_down.is_set_high().unwrap_or_else(|_| true) {
					motor_down.set_low();
					Timer::after_millis(TOGGLE_DELAY).await;
				}
				motor_up.set_high();
			}
			MotorCommand::Down => {
				// info!("motor down");
				if motor_up.is_set_high().unwrap_or_else(|_| true) {
					motor_up.set_low();
					Timer::after_millis(TOGGLE_DELAY).await;
				}
				motor_down.set_high();
			}
		};
		Timer::after_millis(10).await;
	}
}

#[ariel_os::task(autostart, peripherals)]
#[allow(unsafe_code)]
async fn handle_addons(peripherals: pins::AddonPeripherals) {
	Timer::after_millis(25).await;
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

	info!("   initializing addons");
	let mut load = Output::new(peripherals.io_preparation, Level::Low);
	let mut is_high = load.is_set_high().expect("snh");

	info!("   running addons loop");
	loop {
		let preparation = blackboard
			.get::<bool>("preparation")
			.expect("snh");
		if preparation {
			if !is_high {
				load.set_high();
				is_high = true;
			}
		} else {
			if is_high {
				load.set_low();
				is_high = false;
			}
		}
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
async fn handle_upper_end(peripherals: pins::UpperEndPeripherals) {
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

	info!("   initializing upper end");
	let mut led_upper_end = Output::new(peripherals.led_upper_end, Level::Low);

	let mut btn_upper_end = Input::builder(peripherals.btn_upper_end, Pull::Down)
		.build_with_interrupt()
		.unwrap();

	info!("   running upper end loop");
	loop {
		// Wait for end contacts being touched.
		let button = select(btn_upper_end.wait_for_high(), Timer::after_millis(100)).await;
		match button {
			Either::First(_) => {
				// info!("upper end");
				blackboard.set("upper_end", true).expect("snh");
				led_upper_end.set_high();
			}
			Either::Second(_) => {
				// info!("timeout");
				blackboard.set("upper_end", false).expect("snh");
				led_upper_end.set_low();
			}
		}
	}
}

#[ariel_os::task(autostart, peripherals)]
#[allow(unsafe_code)]
async fn handle_lower_end(peripherals: pins::LowerEndPeripherals) {
	Timer::after_millis(11).await;
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

	info!("   initializing lower end");
	let mut led_lower_end = Output::new(peripherals.led_lower_end, Level::Low);

	let mut btn_lower_end = Input::builder(peripherals.btn_lower_end, Pull::Down)
		.build_with_interrupt()
		.unwrap();

	info!("   running lower loop");
	loop {
		// Wait for end contacts being touched.
		let button = select(btn_lower_end.wait_for_high(), Timer::after_millis(100)).await;
		match button {
			Either::First(_) => {
				// info!("lower end");
				blackboard.set("lower_end", true).expect("snh");
				led_lower_end.set_high();
			}
			Either::Second(_) => {
				// info!("timeout");
				blackboard.set("lower_end", false).expect("snh");
				led_lower_end.set_low();
			}
		}
	}
}

#[ariel_os::task(autostart, peripherals)]
#[allow(unsafe_code)]
async fn handle_emergency(peripherals: pins::EmergencyPeripherals) {
	Timer::after_millis(5).await;
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

	info!("   initializing emergency");
	let mut btn_emergency = Input::builder(peripherals.btn_emergency, Pull::Down)
		.build_with_interrupt()
		.unwrap();

	info!("   running emergency loop");
	blackboard.set("emergency", false).expect("snh");
	loop {
		// Wait for emergency button.
		btn_emergency.wait_for_high().await;
		blackboard.set("emergency", true).expect("snh");
		btn_emergency.wait_for_low().await;
		blackboard.set("emergency", false).expect("snh");
	}
}

#[ariel_os::task(autostart)]
async fn main() {
	info!("running garage_door_opener...");
	match behavior().await {
		Ok(_) => {
			Timer::after_millis(100).await;
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
