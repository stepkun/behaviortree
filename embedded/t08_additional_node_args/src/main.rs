// Copyright © 2025 Stephan Kunz
//! Embedded version of [t08_additional_node_args](examples/t08_additional_node_args.rs).

#![no_main]
#![no_std]

use ariel_os::debug::{ExitCode, exit, log::*};
use behaviortree::prelude::*;

const XML: &str = r#"
<root BTCPP_format="4">
    <BehaviorTree ID="MainTree">
        <Sequence>
            <Action_A/>
            <Action_B/>
        </Sequence>
    </BehaviorTree>
</root>
"#;

/// Action `ActionA` has a different constructor than the default one.
#[derive(Action, Debug, Default)]
pub struct ActionA {
	arg1: i32,
	arg2: String,
}

#[async_trait::async_trait]
impl Behavior for ActionA {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		assert_eq!(self.arg1, 42);

		assert_eq!(self.arg2, String::from("hello world"));
		info!(
			"{}: {}, {}",
			behavior.description().name().as_ref(),
			self.arg1,
			self.arg2.as_str()
		);
		Ok(BehaviorState::Success)
	}
}

impl ActionA {
	/// Constructor with arguments.
	#[must_use]
	pub const fn new(arg1: i32, arg2: String) -> Self {
		Self { arg1, arg2 }
	}
}

/// Action `ActionB` implements an initialize(...) method that must be called once at the beginning.
#[derive(Action, Debug, Default)]
pub struct ActionB {
	arg1: i32,
	arg2: String,
}

#[async_trait::async_trait]
impl Behavior for ActionB {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		assert_eq!(self.arg1, 69);
		assert_eq!(self.arg2, String::from("interesting value"));
		info!(
			"{}: {}, {}",
			behavior.description().name().as_ref(),
			self.arg1,
			self.arg2.as_str()
		);
		Ok(BehaviorState::Success)
	}
}

impl ActionB {
	/// Initialization function.
	pub fn initialize(&mut self, arg1: i32, arg2: String) {
		self.arg1 = arg1;
		self.arg2 = arg2;
	}
}

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_core_behaviors()?;

	register_behavior!(factory, ActionA, "Action_A", 42, "hello world".into())?;
	register_behavior!(factory, ActionB, "Action_B")?;

	let mut tree = factory.create_from_text(XML)?;
	drop(factory);

	// initialize ActionB with the help of an iterator
	for node in tree.iter_mut() {
		if node.data().description().name().as_ref() == ("Action_B") {
			if let Some(action) = node
				.behavior_mut()
				.as_any_mut()
				.downcast_mut::<ActionB>()
			{
				action.initialize(69, "interesting value".into());
			}
		}
	}

	let result = tree.tick_while_running().await?;
	Ok(result)
}

#[ariel_os::task(autostart)]
async fn main() {
	info!("running t08_additional_node_args...");
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
