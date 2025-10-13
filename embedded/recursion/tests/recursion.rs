// Copyright © 2025 Stephan Kunz
//! Embedded recursion test. The maximum recursion level depends on stack and memory limits.
//! A reasonable value for mcu's is currently 8!
//! So it is possible to make a Tree with a dept of 8 levels, including sub-trees.

#![no_main]
#![no_std]
#![allow(clippy::unwrap_used)]

extern crate alloc;

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
<!--
									<Sequence>
										<AlwaysSuccess/>
									</Sequence>
-->
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
	use super::*;

	#[test]
	async fn recursion() -> Result<(), Error> {
		let mut factory = BehaviorTreeFactory::new()?;

		let mut tree = factory.create_from_text(XML)?;
		drop(factory);

		tree.tick_while_running().await?;
		Ok(())
	}
}
