// Copyright © 2025 Stephan Kunz
#![allow(missing_docs)]

//! Benchmarks of sequence behaviors [`Sequence`], [`ReactiveSequence`] and [`SequenceWithMemory`]

#[doc(hidden)]
extern crate alloc;

use std::time::Duration;

use behaviortree::{
    behavior::{
        action::ChangeStateAfter, control::{ReactiveSequence, Sequence, SequenceWithMemory}, BehaviorState::{Failure, Running, Success}, BehaviorStatic
    },
    factory::BehaviorTreeFactory,
    register_behavior, SHOULD_NOT_HAPPEN,
};
use criterion::{Criterion, criterion_group, criterion_main};

const SAMPLES: usize = 10;
const ITERATIONS: usize = 10;
const DURATION: Duration = Duration::from_secs(5);

const STANDARD: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="Standard">
	<BehaviorTree ID="Standard">
		<Sequence name="root_sequence">
			<AlwaysSuccess	name="step1"/>
			<AlwaysSuccess/>
			<AlwaysSuccess	name="step2"/>
			<AlwaysSuccess/>
			<AlwaysSuccess	name="step3"/>
			<AlwaysSuccess/>
			<AlwaysSuccess	name="step4"/>
			<AlwaysSuccess/>
			<AlwaysSuccess	name="step5"/>
		</Sequence>
	</BehaviorTree>
</root>
"#;

const REACTIVE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="Reactive">
	<BehaviorTree ID="Reactive">
		<ReactiveSequence name="root_reactive_sequence">
			<AlwaysSuccess	name="step1"/>
			<AlwaysSuccess/>
			<AlwaysSuccess	name="step2"/>
			<AlwaysSuccess/>
			<AlwaysSuccess	name="step3"/>
			<AlwaysSuccess/>
			<AlwaysSuccess	name="step4"/>
			<AlwaysSuccess/>
			<AlwaysSuccess	name="step5"/>
		</ReactiveSequence>
	</BehaviorTree>
</root>
"#;

const WITH_MEMORY: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="WithMemory">
	<BehaviorTree ID="WithMemory">
		<SequenceWithMemory name="root_sequence_with_memory">
			<AlwaysSuccess	name="step1"/>
			<AlwaysSuccess/>
			<AlwaysSuccess	name="step2"/>
			<AlwaysSuccess/>
			<AlwaysSuccess	name="step3"/>
			<AlwaysSuccess/>
			<AlwaysSuccess	name="step4"/>
			<AlwaysSuccess/>
			<AlwaysSuccess	name="step5"/>
		</SequenceWithMemory>
	</BehaviorTree>
</root>
"#;

fn sequence(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .build()
        .expect(SHOULD_NOT_HAPPEN);

    let mut group = c.benchmark_group("sequence");
    group.measurement_time(DURATION).sample_size(SAMPLES);

    let mut factory = BehaviorTreeFactory::default();
    register_behavior!(
        factory,
        ChangeStateAfter,
        "AlwaysFailure",
        Running,
        Failure,
        5
    )
    .expect(SHOULD_NOT_HAPPEN);
    register_behavior!(
        factory,
        ChangeStateAfter,
        "AlwaysSuccess",
        Running,
        Success,
        5
    )
    .expect(SHOULD_NOT_HAPPEN);
    register_behavior!(factory, Sequence, "Sequence").expect(SHOULD_NOT_HAPPEN);
    register_behavior!(factory, ReactiveSequence, "ReactiveSequence").expect(SHOULD_NOT_HAPPEN);
    register_behavior!(factory, SequenceWithMemory, "SequenceWithMemory").expect(SHOULD_NOT_HAPPEN);

    let mut tree = factory.create_from_text(STANDARD).expect(SHOULD_NOT_HAPPEN);
    group.bench_function("standard", |b| {
        b.iter(|| {
            runtime.block_on(async {
                for _ in 1..=ITERATIONS {
                    tree.reset().expect(SHOULD_NOT_HAPPEN);
                    let _result = tree.tick_while_running().await.expect(SHOULD_NOT_HAPPEN);
                }
                std::hint::black_box(());
            });
        });
    });

    let mut tree = factory.create_from_text(REACTIVE).expect(SHOULD_NOT_HAPPEN);
    group.bench_function("reactive", |b| {
        b.iter(|| {
            runtime.block_on(async {
                for _ in 1..=ITERATIONS {
                    tree.reset().expect(SHOULD_NOT_HAPPEN);
                    let _result = tree.tick_while_running().await.expect(SHOULD_NOT_HAPPEN);
                }
                std::hint::black_box(());
            });
        });
    });

    let mut tree = factory.create_from_text(WITH_MEMORY).expect(SHOULD_NOT_HAPPEN);
    group.bench_function("with memory", |b| {
        b.iter(|| {
            runtime.block_on(async {
                for _ in 1..=ITERATIONS {
                    tree.reset().expect(SHOULD_NOT_HAPPEN);
                    let _result = tree.tick_while_running().await.expect(SHOULD_NOT_HAPPEN);
                }
                std::hint::black_box(());
            });
        });
    });
}

criterion_group!(benches, sequence);

criterion_main!(benches);
