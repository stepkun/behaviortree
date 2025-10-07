//! Benchmarks of factory instantiation and tree creation
// Copyright © 2025 Stephan Kunz

#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

use behaviortree::prelude::*;
use criterion::{Criterion, criterion_group, criterion_main};
use std::time::Duration;

const SAMPLES: usize = 10;
const ITERATIONS: usize = 10;
const DURATION: Duration = Duration::from_secs(5);

const TREE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Fallback name="root_fallback">
			<ParallelAll>
				<SubTree ID="subtree" />
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
</root>
"#;

const SUBTREE: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="subtree">
		<Parallel failure_count="3">
			<AlwaysSuccess/>
			<AlwaysFailure/>
			<Sequence>
				<AlwaysSuccess/>
				<Fallback>
					<AlwaysFailure/>
					<Sequence>
						<ReactiveFallback>
							<AlwaysFailure/>
							<AlwaysSuccess/>
						</ReactiveFallback>
						<AlwaysFailure/>
					</Sequence>
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

fn create_factory() -> Result<Box<BehaviorTreeFactory>, Error> {
	let mut factory = BehaviorTreeFactory::new()?;
	factory
		.register_behavior_tree_from_text(SUBTREE)
		.unwrap();
	factory
		.register_behavior_tree_from_text(TREE)
		.unwrap();
	Ok(factory)
}

fn factory(c: &mut Criterion) {
	let mut group = c.benchmark_group("factory");
	group
		.measurement_time(DURATION)
		.sample_size(SAMPLES);

	group.bench_function("instantiation", |b| {
		b.iter(|| {
			for _ in 1..=ITERATIONS {
				let factory = create_factory().unwrap();
				drop(factory);
				std::hint::black_box(());
			}
		});
	});

	let mut factory = create_factory().unwrap();
	group.bench_function("tree creation", |b| {
		b.iter(|| {
			for _ in 1..=100 {
				let _tree = factory.create_tree("MainTree").unwrap();
				std::hint::black_box(());
			}
		});
	});
}

criterion_group!(benches, factory);

criterion_main!(benches);
