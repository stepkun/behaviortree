// Copyright © 2025 Stephan Kunz
#![no_main]
#![no_std]

//! Embedded version of [t09_scripting](examples/t09_scripting.rs).

#[path = "../../common/mod.rs"]
mod common;

use alloc::vec;
use ariel_os::debug::{ExitCode, exit, log::*};
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

#[ariel_os::task(autostart)]
async fn main() {
	info!("running t09_scripting...");
	match example().await {
		Ok(_) => {
			info!("...succeeded!");
			exit(ExitCode::SUCCESS)
		}
		Err(_) => {
			error!("...failed!");
			exit(ExitCode::FAILURE)
		}
	};
}
