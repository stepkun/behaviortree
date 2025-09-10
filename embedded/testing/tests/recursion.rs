// Copyright © 2025 Stephan Kunz
//! Embedded recursion test. The current maximum recursion level is 9!
//! So it is possible to make a Tree with a dept of 9 levels, including sub-trees.

#![no_main]
#![no_std]

extern crate alloc;

// use ariel_os::{
// 	debug::{ExitCode, exit, log::*},
// 	time::Timer,
// };
use behaviortree::prelude::*;

const XML: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="MainTree">
		<Fallback>
			<AlwaysFailure/>
			<Sequence>
				<AlwaysSuccess/>
				<Fallback>
					<AlwaysFailure/>
					<Sequence>
						<AlwaysSuccess/>
						<Fallback>
							<AlwaysFailure/>
							<Sequence>
								<AlwaysSuccess/>
								<Fallback>
									<AlwaysFailure/>
									<Sequence>
										<AlwaysSuccess/>
										<AlwaysSuccess/>
									</Sequence>
									<AlwaysSuccess/>
								</Fallback>
							</Sequence>
						</Fallback>
					</Sequence>
				</Fallback>
			</Sequence>
		</Fallback>
	</BehaviorTree>
</root>
"#;

const XML_FAILS: &str = r#"
<root BTCPP_format="4">
	<BehaviorTree ID="MainTree">
		<Fallback>
			<AlwaysFailure/>
			<Sequence>
				<AlwaysSuccess/>
				<Fallback>
					<AlwaysFailure/>
					<Sequence>
						<AlwaysSuccess/>
						<Fallback>
							<AlwaysFailure/>
							<Sequence>
								<AlwaysSuccess/>
								<Fallback>
									<AlwaysFailure/>
									<Sequence>
										<AlwaysSuccess/>
										<Fallback>
											<AlwaysFailure/>
<!--
											<Sequence>
												<AlwaysSuccess/>
												<Fallback>
													<AlwaysFailure/>
													<Sequence>
														<AlwaysSuccess/>
														<Fallback>
															<AlwaysFailure/>
															<Sequence>
																<AlwaysSuccess/>

															</Sequence>
														</Fallback>
													</Sequence>
												</Fallback>
											</Sequence>
-->																
											<AlwaysSuccess/>
										</Fallback>
									</Sequence>
									<AlwaysSuccess/>
								</Fallback>
							</Sequence>
						</Fallback>
					</Sequence>
				</Fallback>
			</Sequence>
		</Fallback>
	</BehaviorTree>
</root>
"#;

#[cfg(test)]
#[embedded_test::tests]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;

	#[test]
	async fn recursion() {
		let mut factory = BehaviorTreeFactory::default();
		factory.register_test_behaviors().unwrap();

		let mut tree = factory.create_from_text(XML).unwrap();
		drop(factory);

		tree.tick_while_running().await.unwrap();
	}

	#[test]
	#[ignore("as it will fail")]
	async fn recursion_fail() {
		let mut factory = BehaviorTreeFactory::default();
		factory.register_test_behaviors().unwrap();

		let mut tree = factory.create_from_text(XML_FAILS).unwrap();
		drop(factory);

		tree.tick_while_running().await.unwrap();
	}
}
