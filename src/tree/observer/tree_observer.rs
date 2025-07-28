// Copyright © 2025 Stephan Kunz

//! [`BehaviorTreeObserver`] implementation.
//!

extern crate std;

// region:      --- modules
use crate::ConstString;
use alloc::{sync::Arc, vec::Vec};
use parking_lot::Mutex;
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
    /// The shared statistics data
    statistics: Arc<Mutex<Vec<Statistics>>>,
}

impl BehaviorTreeObserver {
    /// Construct a new [`BehaviorTreeObserver`].
    pub fn new(root: &mut BehaviorTree) -> Self {
        let id: ConstString = "statistics".into();
        let statistics: Arc<Mutex<Vec<Statistics>>> = Arc::new(Mutex::new(Vec::new()));

        // add a statistics entry and a callback for each tree element
        for element in root.iter_mut() {
            statistics.lock().push(Statistics::default());
            let statistics_clone: Arc<Mutex<Vec<Statistics>>> = statistics.clone();
			// the callback
            let callback = move |behavior: &BehaviorData, new_state: &mut BehaviorState| {
                let mut stats = statistics_clone.lock();
                let entry = &mut stats[behavior.uid() as usize];
                entry.transitions_count += 1;
                match new_state {
                    BehaviorState::Failure => {
                        entry.failure_count += 1;
                        entry.last_result = *new_state;
                    }
                    BehaviorState::Idle | BehaviorState::Running => {}
                    BehaviorState::Skipped => entry.skip_count += 1,
                    BehaviorState::Success => {
                        entry.success_count += 1;
                        entry.last_result = *new_state;
                    }
                }
                entry.current_state = *new_state;
                // #[cfg(feature = "std")]
                entry.timestamp = Instant::now();
                drop(stats);
            };
            element.add_pre_state_change_callback(id.clone(), callback);
        }
        Self { statistics }
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
