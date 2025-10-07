//! Tests a complex [`BehaviorTree`]
// Copyright © 2025 Stephan Kunz

#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

extern crate alloc;

use behaviortree::prelude::*;

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
	let mut factory = BehaviorTreeFactory::new()?;

	let mut tree = factory.create_from_text(TREE)?;
	drop(factory);

	let mut result = tree.tick_while_running().await?;
	assert_eq!(result, BehaviorState::Success);
	tree.reset().unwrap();
	result = tree.tick_while_running().await?;
	assert_eq!(result, BehaviorState::Success);
	Ok(())
}
