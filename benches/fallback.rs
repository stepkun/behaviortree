//! Benchmarks of fallback behaviors [`Fallback`] and [`ReactiveFallback`]
// Copyright © 2025 Stephan Kunz

#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

use std::time::Duration;

use behaviortree::{
	behavior::{
		BehaviorState::{Failure, Running, Success},
		action::ChangeStateAfter,
		control::{Fallback, ReactiveFallback},
	},
	prelude::*,
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

const ASYNC: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="Async">
	<BehaviorTree ID="Async">
		<AsyncFallback name="root_fallback">
			<AlwaysFailure	name="step1"/>
			<AlwaysFailure/>
			<AlwaysFailure	name="step2"/>
			<AlwaysFailure/>
			<AlwaysFailure	name="step3"/>
			<AlwaysFailure/>
			<AlwaysFailure	name="step4"/>
			<AlwaysFailure/>
			<AlwaysSuccess	name="step5"/>
		</AsyncFallback>
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
		.unwrap();

	let mut group = c.benchmark_group("fallback");
	group
		.measurement_time(DURATION)
		.sample_size(SAMPLES);

	let mut factory = BehaviorTreeFactory::new().unwrap();
	register_behavior!(factory, ChangeStateAfter, "AlwaysFailure", Running, Failure, 5).unwrap();
	register_behavior!(factory, ChangeStateAfter, "AlwaysSuccess", Running, Success, 5).unwrap();
	register_behavior!(factory, ReactiveFallback, "ReactiveFallback").unwrap();
	let bhvr_desc = BehaviorDescription::new(
		"AsyncFallback",
		"AsynchFallback",
		Fallback::kind(),
		true,
		Fallback::provided_ports(),
	);
	let bhvr_creation_fn = Box::new(move || -> Box<dyn BehaviorExecution> { Box::new(Fallback::new(true)) });
	factory
		.registry_mut()
		.add_behavior(bhvr_desc, bhvr_creation_fn)
		.unwrap();

	let mut tree = factory.create_from_text(STANDARD).unwrap();
	group.bench_function("standard", |b| {
		b.iter(|| {
			runtime.block_on(async {
				for _ in 1..=ITERATIONS {
					tree.reset().unwrap();
					let _result = tree.tick_while_running().await.unwrap();
				}
				std::hint::black_box(());
			});
		});
	});

	let mut tree = factory.create_from_text(ASYNC).unwrap();
	group.bench_function("async", |b| {
		b.iter(|| {
			runtime.block_on(async {
				for _ in 1..=ITERATIONS {
					tree.reset().unwrap();
					let _result = tree.tick_while_running().await.unwrap();
				}
				std::hint::black_box(());
			});
		});
	});

	tree = factory.create_from_text(REACTIVE).unwrap();
	group.bench_function("reactive", |b| {
		b.iter(|| {
			runtime.block_on(async {
				for _ in 1..=ITERATIONS {
					tree.reset().unwrap();
					let _result = tree.tick_while_running().await.unwrap();
				}
				std::hint::black_box(());
			});
		});
	});
}

criterion_group!(benches, fallback);

criterion_main!(benches);
