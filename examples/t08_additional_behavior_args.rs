// Copyright © 2025 Stephan Kunz
//! Implements the eigth tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev).
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-basics/tutorial_08_additional_args).
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t08_additional_node_args.cpp).

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

/// Action `ActionA` has a different constructor than the default one, which is generated.
/// We also tell the derive macro not to generate the parameterless registration function.
#[derive(Action, Debug, Default)]
#[behavior(no_register)]
pub struct ActionA {
	#[behavior(parameter)]
	arg1: i32,
	#[behavior(parameter)]
	arg2: String,
}

#[async_trait::async_trait]
impl Behavior for ActionA {
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		assert_eq!(self.arg1, 42);

		assert_eq!(self.arg2, String::from("hello world"));
		println!("{}: {}, {}", behavior.name(), &self.arg1, &self.arg2);
		Ok(BehaviorState::Success)
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
		_children: &mut BehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
		assert_eq!(self.arg1, 69);
		assert_eq!(self.arg2, String::from("interesting value"));
		println!("{}: {}, {}", behavior.name(), &self.arg1, &self.arg2);
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

async fn example() -> Result<(BehaviorState, BehaviorTree), Error> {
	let mut factory = BehaviorTreeFactory::new()?;

	ActionA::register_with(&mut factory, "Action_A", 42, "hello world".into())?;
	ActionB::register(&mut factory, "Action_B")?;

	let mut tree = factory.create_from_text(XML)?;
	drop(factory);

	// initialize ActionB with the help of an iterator
	for behavior in tree.iter_mut() {
		if behavior.name().as_ref() == "Action_B" {
			if let Some(action) = behavior
				.behavior_mut()
				.as_any_mut()
				.downcast_mut::<ActionB>()
			{
				action.initialize(69, "interesting value".into());
			}
		}
	}

	let result = tree.tick_while_running().await?;

	Ok((result, tree))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	example().await?;
	Ok(())
}

#[cfg(test)]
mod test {
	#![allow(missing_docs)]
	#![allow(clippy::unwrap_used)]

	use super::*;

	#[tokio::test]
	async fn t08_additional_behavior_args() -> Result<(), Error> {
		let result = example().await?;
		assert_eq!(result.0, BehaviorState::Success);

		// test the iterator
		let mut iter = result.1.iter();
		assert_eq!(
			iter.next()
				.unwrap()
				.data()
				.description()
				.name()
				.as_ref(),
			"MainTree"
		);
		assert_eq!(
			iter.next()
				.unwrap()
				.data()
				.description()
				.name()
				.as_ref(),
			"Sequence"
		);
		assert_eq!(
			iter.next()
				.unwrap()
				.data()
				.description()
				.name()
				.as_ref(),
			"Action_A"
		);
		assert_eq!(
			iter.next()
				.unwrap()
				.data()
				.description()
				.name()
				.as_ref(),
			"Action_B"
		);
		assert!(iter.next().is_none());

		Ok(())
	}
}
