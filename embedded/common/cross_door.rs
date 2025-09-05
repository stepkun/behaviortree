// Copyright © 2025 Stephan Kunz
//! Cross door behaviors.

#![allow(clippy::unnecessary_wraps)]
#![allow(unused)]

use alloc::sync::Arc;
// use ariel_os::time::Timer;
use behaviortree::prelude::*;
use spin::Mutex;

fn sleep_ms(millisecs: u64) {
	// Timer::after_millis(millisecs).await;
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
			BehaviorKind::Action,
		)?;

		Ok(res)
	}
}
