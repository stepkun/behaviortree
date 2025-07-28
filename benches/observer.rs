// Copyright © 2025 Stephan Kunz
#![allow(missing_docs)]

//! Benchmarks of complex scenario

#[doc(hidden)]
extern crate alloc;

use std::time::Duration;

use behaviortree::{
    BehaviorTreeObserver, Groot2Connector,
    behavior::{
        BehaviorState::{Failure, Running, Success},
        BehaviorStatic,
        action::ChangeStateAfter,
        control::{
            Fallback, Parallel, ParallelAll, ReactiveFallback, ReactiveSequence, Sequence,
            SequenceWithMemory,
        },
    },
    factory::{BehaviorTreeFactory, error::Error},
    register_behavior,
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
        .expect("snh");
    factory.register_behavior_tree_from_text(TREE).expect("snh");
    Ok(factory)
}

fn observer(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .build()
        .expect("snh");
    let mut factory = create_factory().expect("snh");

    let mut group = c.benchmark_group("observer");

    group.measurement_time(DURATION).sample_size(SAMPLES);

    let mut tree = factory.create_tree("MainTree").expect("snh");
    group.bench_function("without", |b| {
        b.iter(|| {
            runtime.block_on(async {
                for _ in 1..=ITERATIONS {
                    tree.reset().await.expect("snh");
                    tree.tick_while_running().await.expect("snh");
                }
                std::hint::black_box(());
            });
            std::hint::black_box(());
        });
    });

    let mut tree = factory.create_tree("MainTree").expect("snh");
    runtime.block_on(async {
        let _observer = BehaviorTreeObserver::new(&mut tree);
    });
    group.bench_function("tree observer", |b| {
        b.iter(|| {
            runtime.block_on(async {
                for _ in 1..=ITERATIONS {
                    tree.reset().await.expect("snh");
                    tree.tick_while_running().await.expect("snh");
                }
                std::hint::black_box(());
            });
            std::hint::black_box(());
        });
    });

    let mut tree = factory.create_tree("MainTree").expect("snh");
    runtime.block_on(async {
        let _publisher = Groot2Connector::new(&mut tree, 9999);
    });
    group.bench_function("groot2", |b| {
        b.iter(|| {
            runtime.block_on(async {
                for _ in 1..=ITERATIONS {
                    tree.reset().await.expect("snh");
                    tree.tick_while_running().await.expect("snh");
                }
                std::hint::black_box(());
            });
            std::hint::black_box(());
        });
    });
}

criterion_group!(benches, observer);

criterion_main!(benches);
