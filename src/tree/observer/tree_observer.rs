// Copyright © 2025 Stephan Kunz

//! [`BehaviorTreeObserver`] implementation.
//!

extern crate std;

// region:      --- modules
use crate::ConstString;
use alloc::{sync::Arc, vec::Vec};
use parking_lot::Mutex;
#[cfg(feature = "std")]
use tokio::sync::mpsc;
#[cfg(feature = "std")]
use tokio::task::JoinHandle;
#[cfg(feature = "std")]
use tokio::time::Instant;

use crate::{
	behavior::{BehaviorData, BehaviorState},
	tree::BehaviorTree,
};
// endregion:   --- modules

// region:      --- Statistics
/// Structure to collect various statistic data.
#[derive(Clone)]
pub struct Statistics {
	/// Last result of a tick, either Success or Failure.
	pub last_result: BehaviorState,
	/// Last state. Can be any state.
	pub current_state: BehaviorState,
	/// count state transitions, excluding transition to Idle.
	pub transitions_count: usize,
	/// count number of transitions to Success.
	pub success_count: usize,
	/// count number of transitions to Failure.
	pub failure_count: usize,
	/// count number of transitions to Skip.
	pub skip_count: usize,
	/// Duration of execution
	#[cfg(feature = "std")]
	pub timestamp: Instant,
}

impl Default for Statistics {
	fn default() -> Self {
		Self {
			last_result: BehaviorState::default(),
			current_state: BehaviorState::default(),
			transitions_count: Default::default(),
			success_count: Default::default(),
			failure_count: Default::default(),
			skip_count: Default::default(),
			timestamp: Instant::now(),
		}
	}
}

impl Statistics {
	fn reset(&mut self) {
		self.last_result = BehaviorState::default();
		self.current_state = BehaviorState::default();
		self.transitions_count = Default::default();
		self.success_count = Default::default();
		self.failure_count = Default::default();
		self.skip_count = Default::default();
		self.timestamp = Instant::now();
	}
}
// endregion:   --- Statistics

// region:      --- BehaviorTreeObserver
/// An observer collecting [`BehaviorTree`] statistics
pub struct BehaviorTreeObserver {
	_handle: JoinHandle<()>,
	statistics: Arc<Mutex<Vec<Statistics>>>,
}

impl BehaviorTreeObserver {
	/// Construct a new [`BehaviorTreeObserver`].
	pub fn new(root: &mut BehaviorTree) -> Self {
		let id: ConstString = "statistics".into();
		let statistics: Arc<Mutex<Vec<Statistics>>> = Arc::new(Mutex::new(Vec::new()));
		let (tx, mut rx) = mpsc::unbounded_channel::<(u16, Instant, BehaviorState, BehaviorState)>();
		// spawn receiver
		let statistics_clone = statistics.clone();
		let handle = tokio::spawn(async move {
			while let Some(val) = rx.recv().await {
				let mut stats = statistics_clone.lock();
				let entry = &mut stats[val.0 as usize];
				if val.3 != val.2 {
					entry.transitions_count += 1;
					match val.3 {
						BehaviorState::Failure => {
							entry.failure_count += 1;
							entry.last_result = val.3;
						}
						BehaviorState::Idle | BehaviorState::Running => {}
						BehaviorState::Skipped => entry.skip_count += 1,
						BehaviorState::Success => {
							entry.success_count += 1;
							entry.last_result = val.3;
						}
					}
					entry.current_state = val.3;
					entry.timestamp = val.1;
				}
				drop(stats);
			}
		});

		// add a statistics entry and a callback for each tree element
		for element in root.iter_mut() {
			statistics.lock().push(Statistics::default());
			let tx_clone = tx.clone();
			let callback = move |behavior: &BehaviorData, new_state: &mut BehaviorState| {
				let old_state = behavior.state();
				if old_state != *new_state {
					let timestamp = Instant::now();
					let tuple = (behavior.uid(), timestamp, behavior.state(), *new_state);
					// ignore any errors when sending
					let _ = tx_clone.send(tuple);
				}
			};
			element.add_pre_state_change_callback(id.clone(), callback);
		}
		Self {
			_handle: handle,
			statistics,
		}
	}

	/// Get the [`Statistics`] for a [`BehaviorTreeElement`](crate::tree::BehaviorTreeElement) using its uid.
	#[must_use]
	pub fn get_statistics(&self, uid: u16) -> Option<Statistics> {
		if self.statistics.lock().len() >= uid as usize {
			return Some((self.statistics.lock()[uid as usize]).clone());
		}
		None
	}

	/// Reset the [`BehaviorTreeObserver`].
	pub fn reset(&self) {
		for stats in &mut (*self.statistics.lock()) {
			stats.reset();
		}
	}
}
// endregion:   --- BehaviorTreeObserver
