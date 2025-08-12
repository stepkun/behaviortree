// Copyright © 2025 Stephan Kunz
#![allow(missing_docs)]

//! Benchmarks of complex scenario

#[doc(hidden)]
extern crate alloc;

use std::time::Duration;

use behaviortree::{
	BehaviorTreeObserver, Groot2Connector, SHOULD_NOT_HAPPEN,
	behavior::{
		Behavior,
		BehaviorState::{Failure, Running, Success},
		BehaviorStatic,
		action::ChangeStateAfter,
		control::{ParallelAll, ReactiveFallback, ReactiveSequence, SequenceWithMemory},
	},
	factory::{BehaviorTreeFactory, error::Error},
	register_behavior,
};
use criterion::{Criterion, criterion_group, criterion_main};
use tokio::try_join;

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

const TREE1: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree1">
	<BehaviorTree ID="MainTree1">
		<Sequence name="root_sequence">
			<AlwaysFailure/>
			<SubTree ID="subtree"/>
			<AlwaysSuccess/>
		</Sequence>
	</BehaviorTree>
</root>
"#;

const TREE2: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree2">
	<BehaviorTree ID="MainTree2">
		<Fallback name="root_fallback">
			<AlwaysFailure/>
			<SubTree ID="subtree"/>
		</Fallback>
	</BehaviorTree>
</root>
"#;

const TREE3: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree3">
	<BehaviorTree ID="MainTree3">
		<ParallelAll name="root_parallel">
			<ReactiveSequence>
				<AlwaysSuccess/>
				<AlwaysSuccess/>
				<AlwaysSuccess/>
			</ReactiveSequence>
			<ReactiveFallback>
				<AlwaysFailure/>
				<AlwaysFailure/>
				<AlwaysSuccess/>
			</ReactiveFallback>
			<SubTree ID="subtree"/>
			<Sequence>
				<AlwaysSuccess/>
				<AlwaysSuccess/>
				<AlwaysSuccess/>
				<AlwaysSuccess/>
				<AlwaysSuccess/>
			</Sequence>
		</ParallelAll>
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
	register_behavior!(factory, ChangeStateAfter, "AlwaysFailure", Running, Failure, 5)?;
	register_behavior!(factory, ChangeStateAfter, "AlwaysSuccess", Running, Success, 5)?;
	register_behavior!(factory, ParallelAll, "ParallelAll")?;
	register_behavior!(factory, ReactiveFallback, "ReactiveFallback")?;
	register_behavior!(factory, ReactiveSequence, "ReactiveSequence")?;
	register_behavior!(factory, SequenceWithMemory, "SequenceWithMemory")?;
	factory
		.register_behavior_tree_from_text(SUBTREE)
		.expect(SHOULD_NOT_HAPPEN);
	factory
		.register_behavior_tree_from_text(TREE)
		.expect(SHOULD_NOT_HAPPEN);
	factory
		.register_behavior_tree_from_text(TREE1)
		.expect(SHOULD_NOT_HAPPEN);
	factory
		.register_behavior_tree_from_text(TREE2)
		.expect(SHOULD_NOT_HAPPEN);
	factory
		.register_behavior_tree_from_text(TREE3)
		.expect(SHOULD_NOT_HAPPEN);
	Ok(factory)
}

#[allow(clippy::too_many_lines)]
fn trees(c: &mut Criterion) {
	let runtime = tokio::runtime::Builder::new_multi_thread()
		.enable_io()
		.build()
		.expect(SHOULD_NOT_HAPPEN);

	let mut group = c.benchmark_group("tree");
	group
		.measurement_time(DURATION)
		.sample_size(SAMPLES);

	let mut factory = create_factory().expect(SHOULD_NOT_HAPPEN);
	let mut tree = factory
		.create_tree("MainTree")
		.expect(SHOULD_NOT_HAPPEN);
	group.bench_function("execution", |b| {
		b.iter(|| {
			runtime.block_on(async {
				for _ in 1..=ITERATIONS {
					tree.reset().expect(SHOULD_NOT_HAPPEN);
					tree.tick_while_running()
						.await
						.expect(SHOULD_NOT_HAPPEN);
				}
				std::hint::black_box(());
			});
		});
	});

	let mut tree = factory
		.create_tree("MainTree")
		.expect(SHOULD_NOT_HAPPEN);
	runtime.block_on(async {
		let _observer = BehaviorTreeObserver::new(&mut tree);
	});
	group.bench_function("tree observer", |b| {
		b.iter(|| {
			runtime.block_on(async {
				for _ in 1..=ITERATIONS {
					tree.reset().expect(SHOULD_NOT_HAPPEN);
					tree.tick_while_running()
						.await
						.expect(SHOULD_NOT_HAPPEN);
				}
				std::hint::black_box(());
			});
			std::hint::black_box(());
		});
	});

	let mut tree = factory
		.create_tree("MainTree")
		.expect(SHOULD_NOT_HAPPEN);
	runtime.block_on(async {
		let _publisher = Groot2Connector::new(&mut tree, 9999);
	});
	group.bench_function("groot2 connector", |b| {
		b.iter(|| {
			runtime.block_on(async {
				for _ in 1..=ITERATIONS {
					tree.reset().expect(SHOULD_NOT_HAPPEN);
					tree.tick_while_running()
						.await
						.expect(SHOULD_NOT_HAPPEN);
				}
				std::hint::black_box(());
			});
			std::hint::black_box(());
		});
	});

	group.bench_function("multi concurrent", |b| {
		b.iter(|| {
			let mut tree = factory
				.create_tree("MainTree")
				.expect(SHOULD_NOT_HAPPEN);
			let mut tree1 = factory
				.create_tree("MainTree1")
				.expect(SHOULD_NOT_HAPPEN);
			let mut tree2 = factory
				.create_tree("MainTree2")
				.expect(SHOULD_NOT_HAPPEN);
			let mut tree3 = factory
				.create_tree("MainTree3")
				.expect(SHOULD_NOT_HAPPEN);
			runtime.block_on(async {
				for _ in 1..=ITERATIONS {
					let h = async {
						tree.reset()?;
						tree.tick_while_running().await
					};
					let h1 = async {
						tree1.reset()?;
						tree1.tick_while_running().await
					};
					let h2 = async {
						tree2.reset()?;
						tree2.tick_while_running().await
					};
					let h3 = async {
						tree3.reset()?;
						tree3.tick_while_running().await
					};
					try_join!(h, h1, h2, h3).expect(SHOULD_NOT_HAPPEN);
				}
				std::hint::black_box(());
			});
		});
	});

	group.bench_function("multi spawned", |b| {
		b.iter(|| {
			let mut tree = factory
				.create_tree("MainTree")
				.expect(SHOULD_NOT_HAPPEN);
			let mut tree1 = factory
				.create_tree("MainTree1")
				.expect(SHOULD_NOT_HAPPEN);
			let mut tree2 = factory
				.create_tree("MainTree2")
				.expect(SHOULD_NOT_HAPPEN);
			let mut tree3 = factory
				.create_tree("MainTree3")
				.expect(SHOULD_NOT_HAPPEN);
			runtime.block_on(async {
				for _ in 1..=ITERATIONS {
					let h = tokio::spawn(async {
						tree.reset().expect(SHOULD_NOT_HAPPEN);
						tree.tick_while_running()
							.await
							.expect(SHOULD_NOT_HAPPEN);
						tree
					});
					let h1 = tokio::spawn(async {
						tree1.reset().expect(SHOULD_NOT_HAPPEN);
						tree1
							.tick_while_running()
							.await
							.expect(SHOULD_NOT_HAPPEN);
						tree1
					});
					let h2 = tokio::spawn(async {
						tree2.reset().expect(SHOULD_NOT_HAPPEN);
						tree2
							.tick_while_running()
							.await
							.expect(SHOULD_NOT_HAPPEN);
						tree2
					});
					let h3 = tokio::spawn(async {
						tree3.reset().expect(SHOULD_NOT_HAPPEN);
						tree3
							.tick_while_running()
							.await
							.expect(SHOULD_NOT_HAPPEN);
						tree3
					});
					(tree, tree1, tree2, tree3) = try_join!(h, h1, h2, h3).expect(SHOULD_NOT_HAPPEN);
				}
				std::hint::black_box(());
			});
		});
	});
}

criterion_group!(benches, trees);

criterion_main!(benches);
