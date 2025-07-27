// Copyright © 2025 Stephan Kunz

//! Run all tests for behaviortree macros

#[test]
fn tests() {
	let t = trybuild::TestCases::new();
	t.pass("tests/behavior/01-usage.rs");
	t.compile_fail("tests/behavior/02-wrong-usage.rs");
	t.compile_fail("tests/behavior/03-wrong-usage.rs");
}
