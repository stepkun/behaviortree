// Copyright © 2025 Stephan Kunz
#![allow(missing_docs)]

//! Benchmarks of complex scenario

#[doc(hidden)]
extern crate alloc;

use std::time::Duration;

use behaviortree::{
    behavior::{
        action::ChangeStateAfter, control::{
            Fallback, Parallel, ParallelAll, ReactiveFallback, ReactiveSequence, Sequence,
            SequenceWithMemory,
        }, BehaviorState::{Failure, Running, Success}, BehaviorStatic
    },
    factory::{error::Error, BehaviorTreeFactory},
    register_behavior, SHOULD_NOT_HAPPEN,
};
use criterion::{Criterion, criterion_group, criterion_main};

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

fn create_factory() -> Result<BehaviorTreeFactory, Error> {
    let mut factory = BehaviorTreeFactory::default();
    register_behavior!(
        factory,
        ChangeStateAfter,
        "AlwaysFailure",
        Running,
        Failure,
        5
    )?;
    register_behavior!(
        factory,
        ChangeStateAfter,
        "AlwaysSuccess",
        Running,
        Success,
        5
    )?;
    register_behavior!(factory, Fallback, "Fallback")?;
    register_behavior!(factory, Parallel, "Parallel")?;
    register_behavior!(factory, ParallelAll, "ParallelAll")?;
    register_behavior!(factory, ReactiveFallback, "ReactiveFallback")?;
    register_behavior!(factory, ReactiveSequence, "ReactiveSequence")?;
    register_behavior!(factory, Sequence, "Sequence")?;
    register_behavior!(factory, SequenceWithMemory, "SequenceWithMemory")?;
    factory
        .register_behavior_tree_from_text(SUBTREE)
        .expect(SHOULD_NOT_HAPPEN);
    factory.register_behavior_tree_from_text(TREE).expect(SHOULD_NOT_HAPPEN);
    Ok(factory)
}

fn factory(c: &mut Criterion) {
    let mut group = c.benchmark_group("factory");
    group.measurement_time(DURATION).sample_size(SAMPLES);

    group.bench_function("instantiation", |b| {
        b.iter(|| {
            for _ in 1..=ITERATIONS {
                let factory = create_factory().expect(SHOULD_NOT_HAPPEN);
                drop(factory);
            }
            std::hint::black_box(());
        });
    });

    let mut factory = create_factory().expect(SHOULD_NOT_HAPPEN);
    group.bench_function("tree creation", |b| {
        b.iter(|| {
            for _ in 1..=100 {
                let _tree = factory.create_tree("MainTree").expect(SHOULD_NOT_HAPPEN);
            }
            std::hint::black_box(());
        });
    });
}

criterion_group!(benches, factory);

criterion_main!(benches);
