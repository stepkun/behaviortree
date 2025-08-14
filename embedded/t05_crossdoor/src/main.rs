// Copyright © 2025 Stephan Kunz
#![no_main]
#![no_std]

//! Embedded version of [t05_crossdoor](examples/t05_crossdoor.rs)

use alloc::sync::Arc;
use ariel_os::{
	debug::{ExitCode, exit, log::*},
	time::{Duration, Timer},
};
use behaviortree::prelude::*;

const XML: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="CrossDoor">
        <Sequence>
            <Fallback>
                <Inverter>
                    <IsDoorClosed/>
                </Inverter>
                <SubTree ID="DoorClosed"/>
            </Fallback>
            <PassThroughDoor/>
        </Sequence>
	</BehaviorTree>

    <BehaviorTree ID="DoorClosed">
        <Fallback>
            <OpenDoor/>
            <RetryUntilSuccessful num_attempts="5">
                <PickLock/>
            </RetryUntilSuccessful>
            <SmashDoor/>
        </Fallback>
    </BehaviorTree>
</root>
"#;

fn sleep_ms(millisecs: u64) {
	//Timer::after_millis(100).await;
	thread::sleep(Duration::from_millis(millisecs));
}

/// `CrossDoor` behavior interface
pub struct CrossDoor {
	door_open: bool,
	door_locked: bool,
	pick_attempts: u8,
	needed_attempts: u8,
}

impl Default for CrossDoor {
	fn default() -> Self {
		Self {
			door_open: false,
			door_locked: true,
			pick_attempts: 0,
			needed_attempts: 3,
		}
	}
}

impl CrossDoor {
	/// SUCCESS if `door_open` == true
	/// # Errors
	/// never
	pub fn is_door_closed(&self) -> BehaviorResult {
		sleep_ms(200);
		if self.door_open {
			Ok(BehaviorState::Failure)
		} else {
			Ok(BehaviorState::Success)
		}
	}

	/// FAILURE if `door_locked` == true
	/// # Errors
	/// never
	pub fn open_door(&mut self) -> BehaviorResult {
		sleep_ms(500);
		if self.door_locked {
			Ok(BehaviorState::Failure)
		} else {
			self.door_open = true;
			Ok(BehaviorState::Success)
		}
	}

	/// SUCCESS if `door_open` == true
	/// # Errors
	/// never
	pub fn pass_through_door(&self) -> BehaviorResult {
		sleep_ms(500);
		if self.door_open {
			Ok(BehaviorState::Success)
		} else {
			Ok(BehaviorState::Failure)
		}
	}

	/// May open a locked door.
	/// The number of needed attempts is randomly set
	/// # Errors
	/// never
	pub fn pick_lock(&mut self) -> BehaviorResult {
		sleep_ms(500);
		self.pick_attempts += 1;
		// succeed at random attempt
		if self.pick_attempts >= self.needed_attempts {
			self.door_locked = false;
			self.door_open = true;
			Ok(BehaviorState::Success)
		} else {
			Ok(BehaviorState::Failure)
		}
	}

	/// Reset `CrossDoor`
	pub fn reset(&mut self) {
		self.door_open = false;
		self.door_locked = true;
		self.pick_attempts = 0;
		self.needed_attempts = 3;
	}

	/// Will always open a door
	/// # Errors
	/// never
	pub const fn smash_door(&mut self) -> BehaviorResult {
		self.door_locked = false;
		self.door_open = true;
		// smash always works
		Ok(BehaviorState::Success)
	}

	/// Registration function for the `CrossDoor` interface
	/// # Errors
	pub fn register_behaviors(factory: &mut BehaviorTreeFactory) -> Result<Arc<Mutex<Self>>, Error> {
		let res = register_behavior!(
			factory,
			Self::default(),
			is_door_closed,
			"IsDoorClosed",
			BehaviorKind::Condition,
			open_door,
			"OpenDoor",
			BehaviorKind::Action,
			pass_through_door,
			"PassThroughDoor",
			BehaviorKind::Action,
			pick_lock,
			"PickLock",
			BehaviorKind::Action,
			smash_door,
			"SmashDoor",
			BehaviorKind::Condition,
		)?;

		Ok(res)
	}
}

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_groot2_behaviors()?;

	CrossDoor::register_behaviors(&mut factory)?;

	// In this example a single XML contains multiple <BehaviorTree>
	// To determine which one is the "main one", we should first register
	// the XML and then allocate a specific tree, using its ID
	factory.register_behavior_tree_from_text(XML)?;
	let mut tree = factory.create_tree("CrossDoor")?;
	drop(factory);

	let result = tree.tick_while_running().await?;
	Ok(result)
}

#[ariel_os::task(autostart)]
async fn main() {
	info!("running t05_crossdoor...");
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
