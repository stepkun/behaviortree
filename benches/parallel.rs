// Copyright © 2025 Stephan Kunz
#![allow(missing_docs)]

//! Benchmarks of parallel behaviors [`Parallel`] and [`ParallelAll`]

#[doc(hidden)]
extern crate alloc;

use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use behaviortree::{
	behavior::{
		BehaviorState::{Running, Success},
		BehaviorStatic,
		action::ChangeStateAfter,
		control::{Parallel, ParallelAll, Sequence},
	},
	factory::BehaviorTreeFactory,
	register_behavior,
};

const SAMPLES: usize = 10;
const ITERATIONS: usize = 10;
const DURATION: Duration = Duration::from_secs(5);

const STANDARD: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="Standard">
	<BehaviorTree ID="Standard">
		<Parallel name="root_parallel" failure_count="-1" success_count="5">
			<SubTree ID="SubSequence" name="step1"/>
			<SubTree ID="SubSequence" name="step2"/>
			<SubTree ID="SubSequence" name="step3"/>
			<SubTree ID="SubSequence" name="step4"/>
			<SubTree ID="SubSequence" name="step5"/>
		</Parallel>
	</BehaviorTree>
</root>
"#;

const ALL: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="All">
	<BehaviorTree ID="All">
		<ParallelAll name="root_parallel_all">
			<SubTree ID="SubSequence" name="step1"/>
			<SubTree ID="SubSequence" name="step2"/>
			<SubTree ID="SubSequence" name="step3"/>
			<SubTree ID="SubSequence" name="step4"/>
			<SubTree ID="SubSequence" name="step5"/>
		</ParallelAll>
	</BehaviorTree>
</root>
"#;

const SUBTREE: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="SubSequence">
		<Sequence>
			<AlwaysSuccess	name="sub_step1"/>
			<AlwaysSuccess/>
			<AlwaysSuccess	name="sub_step2"/>
			<AlwaysSuccess/>
			<AlwaysSuccess	name="sub_step3"/>
			<AlwaysSuccess/>
			<AlwaysSuccess	name="sub_step4"/>
			<AlwaysSuccess/>
			<AlwaysSuccess	name="sub_step5"/>
		</Sequence>
	</BehaviorTree>
</root>
"#;

fn parallel(c: &mut Criterion) {
	let runtime = tokio::runtime::Builder::new_multi_thread()
		.build()
		.expect("snh");

	let mut group = c.benchmark_group("parallel");
	group
		.measurement_time(DURATION)
		.sample_size(SAMPLES);

	let mut factory = BehaviorTreeFactory::default();
	register_behavior!(factory, ChangeStateAfter, "AlwaysSuccess", Running, Success, 5).expect("snh");
	register_behavior!(factory, Parallel, "Parallel").expect("snh");
	register_behavior!(factory, ParallelAll, "ParallelAll").expect("snh");
	register_behavior!(factory, Sequence, "Sequence").expect("snh");
	factory
		.register_behavior_tree_from_text(SUBTREE)
		.expect("snh");

	let mut tree = factory.create_from_text(STANDARD).expect("snh");
	group.bench_function("standard", |b| {
		b.iter(|| {
			runtime.block_on(async {
				for _ in 1..=ITERATIONS {
					tree.reset().await.expect("snh");
					let _result = tree.tick_while_running().await.expect("snh");
				}
				std::hint::black_box(());
			});
		});
	});

	let mut tree = factory.create_from_text(ALL).expect("snh");
	group.bench_function("all", |b| {
		b.iter(|| {
			runtime.block_on(async {
				for _ in 1..=ITERATIONS {
					tree.reset().await.expect("snh");
					let _result = tree.tick_while_running().await.expect("snh");
				}
				std::hint::black_box(());
			});
		});
	});
}

criterion_group!(benches, parallel);

criterion_main!(benches);
