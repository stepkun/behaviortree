// Copyright © 2025 Stephan Kunz

//! This test implements the nineth tutorial/example from [BehaviorTree.CPP](https://www.behaviortree.dev)
//!
//! [tutorial:](https://www.behaviortree.dev/docs/tutorial-basics/tutorial_09_scripting)
//! [cpp-source:](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/examples/t09_scripting.cpp)
//!

#[doc(hidden)]
extern crate alloc;
mod common;

use behaviortree::prelude::*;
use common::test_data::SaySomething;

const XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<root BTCPP_format="4">
    <BehaviorTree ID="MainTree">
        <Sequence>
            <Script code=" msg:='hello world' " />
            <Script code=" A:=THE_ANSWER; B:=3.14; color:=RED " />
			<!-- the original '&&' is a none valid xml, so it is replaced by '&amp;&amp;' -->
            <Precondition if="A>-B &amp;&amp; color != BLUE" else="FAILURE">
                <Sequence>
                  <SaySomething message="{A}"/>
                  <SaySomething message="{B}"/>
                  <SaySomething message="{msg}"/>
                  <SaySomething message="{color}"/>
                </Sequence>
            </Precondition>
        </Sequence>
    </BehaviorTree>
</root>
"#;

#[derive(ScriptEnum)]
#[allow(unused, clippy::upper_case_acronyms)]
enum Color {
	RED = 1,
	BLUE,
	GREEN = 4,
}

async fn example() -> BehaviorTreeResult {
	let mut factory = BehaviorTreeFactory::with_groot2_behaviors()?;

	register_scripting_enum!(factory, Color);
	register_scripting_enum!(factory, "THE_ANSWER", 42, "OTHER", 43);

	register_behavior!(factory, SaySomething, "SaySomething")?;

	let mut tree = factory.create_from_text(XML)?;
	drop(factory);

	let result = tree.tick_while_running().await?;
	Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	let result = example().await?;
	assert_eq!(result, BehaviorState::Success);
	Ok(())
}

#[cfg(test)]
mod test {
	use super::*;

	#[tokio::test]
	async fn t09_scripting() -> Result<(), Error> {
		let result = example().await?;
		assert_eq!(result, BehaviorState::Success);
		Ok(())
	}
}
