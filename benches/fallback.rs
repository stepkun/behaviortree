// Copyright © 2025 Stephan Kunz
#![allow(missing_docs)]

//! Benchmarks of Fallback behaviors [`Fallback`] and [`ReactiveFallback`]

#[doc(hidden)]
extern crate alloc;

use std::time::Duration;

use behaviortree::{
    SHOULD_NOT_HAPPEN,
    behavior::{
        Behavior,
        BehaviorState::{Failure, Running, Success},
        BehaviorStatic,
        action::ChangeStateAfter,
        control::ReactiveFallback,
    },
    factory::BehaviorTreeFactory,
    register_behavior,
};
use criterion::{Criterion, criterion_group, criterion_main};

const SAMPLES: usize = 10;
const ITERATIONS: usize = 10;
const DURATION: Duration = Duration::from_secs(5);

const STANDARD: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="Standard">
	<BehaviorTree ID="Standard">
		<Fallback name="root_fallback">
			<AlwaysFailure	name="step1"/>
			<AlwaysFailure/>
			<AlwaysFailure	name="step2"/>
			<AlwaysFailure/>
			<AlwaysFailure	name="step3"/>
			<AlwaysFailure/>
			<AlwaysFailure	name="step4"/>
			<AlwaysFailure/>
			<AlwaysSuccess	name="step5"/>
		</Fallback>
	</BehaviorTree>
</root>
"#;

const REACTIVE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="Reactive">
	<BehaviorTree ID="Reactive">
		<ReactiveFallback name="root_reactive_fallback">
			<AlwaysFailure	name="step1"/>
			<AlwaysFailure/>
			<AlwaysFailure	name="step2"/>
			<AlwaysFailure/>
			<AlwaysFailure	name="step3"/>
			<AlwaysFailure/>
			<AlwaysFailure	name="step4"/>
			<AlwaysFailure/>
			<AlwaysFailure	name="step5"/>
		</ReactiveFallback>
	</BehaviorTree>
</root>
"#;

fn fallback(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .build()
        .expect(SHOULD_NOT_HAPPEN);

    let mut group = c.benchmark_group("fallback");
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
    register_behavior!(factory, ReactiveFallback, "ReactiveFallback").expect(SHOULD_NOT_HAPPEN);

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

    tree = factory.create_from_text(REACTIVE).expect(SHOULD_NOT_HAPPEN);
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
}

criterion_group!(benches, fallback);

criterion_main!(benches);
