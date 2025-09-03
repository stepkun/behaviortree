//! Tests the [`BehaviorTreeObserver`]
// Copyright © 2025 Stephan Kunz

#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

extern crate alloc;

use behaviortree::prelude::*;
use behaviortree::{BehaviorTreeObserver, behavior::action::ChangeStateAfter};

const TREE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Fallback name="observer">
			<AlwaysFailure	name="step1"/>
			<AlwaysFailure	name="step2"/>
			<AlwaysSuccess	name="step3"/>
		</Fallback>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn tree_observer() -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(
		factory,
		ChangeStateAfter,
		"AlwaysFailure",
		BehaviorState::Running,
		BehaviorState::Failure,
		2
	)
	.unwrap();
	register_behavior!(
		factory,
		ChangeStateAfter,
		"AlwaysSuccess",
		BehaviorState::Running,
		BehaviorState::Success,
		2
	)
	.unwrap();

	let mut tree = factory.create_from_text(TREE)?;
	let observer = BehaviorTreeObserver::new(&mut tree);
	drop(factory);

	let result = tree.tick_while_running().await?;
	assert_eq!(result, BehaviorState::Success);
	assert_eq!(
		observer
			.get_statistics(4)
			.unwrap()
			.transitions_count,
		3
	);
	assert_eq!(
		observer
			.get_statistics(4)
			.unwrap()
			.transitions_count,
		3
	);
	assert_eq!(
		observer
			.get_statistics(0)
			.unwrap()
			.transitions_count,
		2
	);
	Ok(())
}
