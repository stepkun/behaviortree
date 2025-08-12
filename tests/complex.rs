// Copyright © 2025 Stephan Kunz

//! Tests a complex [`BehaviorTree`]

extern crate alloc;

use behaviortree::prelude::*;
use behaviortree::{
	SHOULD_NOT_HAPPEN,
	behavior::{
		action::ChangeStateAfter,
		control::{ParallelAll, ReactiveFallback, ReactiveSequence, SequenceWithMemory},
	},
};

const TREE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Fallback name="root_fallback">
			<ParallelAll>
				<Sequence>
					<AlwaysSuccess/>
					<Fallback>
						<AlwaysFailure/>
						<AlwaysFailure/>
						<AlwaysFailure/>
						<AlwaysSuccess/>
					</Fallback>
					<AlwaysSuccess/>
				</Sequence>
				<ReactiveSequence>
					<AlwaysSuccess/>
					<Fallback>
						<AlwaysFailure/>
						<AlwaysSuccess/>
					</Fallback>
					<AlwaysSuccess/>
				</ReactiveSequence>
				<SequenceWithMemory>
					<AlwaysSuccess/>
					<ReactiveFallback>
						<AlwaysFailure/>
						<AlwaysSuccess/>
					</ReactiveFallback>
					<AlwaysSuccess/>
				</SequenceWithMemory>
			</ParallelAll>
		</Fallback>
	</BehaviorTree>

	<BehaviorTree ID="subtree1">
		<Parallel failure_count="3">
			<AlwaysSuccess/>
			<AlwaysFailure/>
			<Sequence>
				<AlwaysSuccess/>
				<Fallback>
					<AlwaysFailure/>
					<ReactiveSequence>
						<ReactiveFallback>
							<AlwaysFailure/>
							<AlwaysSuccess/>
						</ReactiveFallback>
						<AlwaysFailure/>
					</ReactiveSequence>
					<AlwaysSuccess/>
				</Fallback>
				<AlwaysSuccess/>
			</Sequence>
			<AlwaysSuccess/>
			<AlwaysFailure/>
		</Parallel>
	</BehaviorTree>
</root>
"#;

#[tokio::test]
async fn complex() -> Result<(), Error> {
	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(
		factory,
		ChangeStateAfter,
		"AlwaysFailure",
		BehaviorState::Running,
		BehaviorState::Failure,
		5
	)
	.expect(SHOULD_NOT_HAPPEN);
	register_behavior!(
		factory,
		ChangeStateAfter,
		"AlwaysSuccess",
		BehaviorState::Running,
		BehaviorState::Success,
		5
	)
	.expect(SHOULD_NOT_HAPPEN);
	register_behavior!(factory, ParallelAll, "ParallelAll").expect(SHOULD_NOT_HAPPEN);
	register_behavior!(factory, ReactiveFallback, "ReactiveFallback").expect(SHOULD_NOT_HAPPEN);
	register_behavior!(factory, ReactiveSequence, "ReactiveSequence").expect(SHOULD_NOT_HAPPEN);
	register_behavior!(factory, SequenceWithMemory, "SequenceWithMemory").expect(SHOULD_NOT_HAPPEN);

	let mut tree = factory.create_from_text(TREE)?;
	drop(factory);

	let mut result = tree.tick_while_running().await?;
	assert_eq!(result, BehaviorState::Success);
	tree.reset().expect(SHOULD_NOT_HAPPEN);
	result = tree.tick_while_running().await?;
	assert_eq!(result, BehaviorState::Success);
	Ok(())
}
