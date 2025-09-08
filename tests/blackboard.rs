//! Tests for [`BlackboardData`] and [`SharedBlackboard`]/[`Blackboard`](behaviortree::Blackboard)
// Copyright © 2025 Stephan Kunz

#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

use behaviortree::{BlackboardData, BlackboardInterface, Remappings, SharedBlackboard};

#[test]
fn blackboard() {
	let mut blackboard = BlackboardData::default();

	let value = blackboard.get::<i32>("test");
	assert!(value.is_err());

	let old = blackboard
		.set("test", String::from("test"))
		.unwrap();
	assert_eq!(old, None);

	let value = blackboard.get::<String>("test").unwrap();
	assert_eq!(value, String::from("test"));

	let old = blackboard
		.set("test", String::from("changed"))
		.unwrap();
	assert_eq!(old, Some(String::from("test")));

	let value = blackboard.get::<String>("test").unwrap();
	assert_eq!(value, String::from("changed"));

	let value = blackboard.set("test", 42);
	assert!(value.is_err());

	let old = blackboard.delete::<String>("test").unwrap();
	assert_eq!(old, String::from("changed"));

	let old = blackboard.set("test", 42).unwrap();
	assert_eq!(old, None);

	let value = blackboard.get::<i32>("test").unwrap();
	assert_eq!(value, 42);
}

#[test]
fn blackboard_node_default() {
	let mut level0 = SharedBlackboard::new("level0");

	let value = level0.get::<i32>("test");
	assert!(value.is_err());

	let old = level0.set("test", String::from("test")).unwrap();
	assert_eq!(old, None);

	let value = level0.get::<String>("test").unwrap();
	assert_eq!(value, String::from("test"));

	let old = level0.set("test", 42);
	assert!(old.is_err());

	let old = level0.delete::<String>("test").unwrap();
	assert_eq!(old, String::from("test"));

	let old = level0.set("test", 42).unwrap();
	assert_eq!(old, None);

	let value = level0.get::<i32>("test").unwrap();
	assert_eq!(value, 42);
}

#[test]
fn blackboard_node_with_parent() {
	let mut level0 = SharedBlackboard::new("level0");

	let old = level0
		.set("test1", String::from("test1"))
		.unwrap();
	assert_eq!(old, None);
	let old = level0
		.set("test2", String::from("test2"))
		.unwrap();
	assert_eq!(old, None);

	let mut remappings = Remappings::default();
	remappings.add("test", "test1").unwrap();
	let mut node = SharedBlackboard::with_parent("level0", level0, remappings, true);

	let old = node.set("@other", String::from("other")).unwrap();
	assert_eq!(old, None);

	let old = node.set("test3", String::from("test3")).unwrap();
	assert_eq!(old, None);

	let value = node.get::<String>("@other").unwrap();
	assert_eq!(value, String::from("other"));
	let value = node.get::<String>("test").unwrap();
	assert_eq!(value, String::from("test1"));
	let value = node.get::<String>("test2").unwrap();
	assert_eq!(value, String::from("test2"));
	let value = node.get::<String>("test3").unwrap();
	assert_eq!(value, String::from("test3"));
}

#[test]
fn blackboard_node_hierarchy() {
	let mut level0 = SharedBlackboard::new("level0");

	let mut remappings1 = Remappings::default();
	remappings1.add("levelB", "levelA").unwrap();
	let mut level1 = SharedBlackboard::with_parent("level1", level0.clone(), remappings1, true);

	let mut remappings2 = Remappings::default();
	remappings2.add("levelC", "levelB").unwrap();
	let mut level2 = SharedBlackboard::with_parent("level2", level1.clone(), remappings2, true);

	let mut remappings3 = Remappings::default();
	remappings3.add("levelD", "levelC").unwrap();
	let mut level3 = SharedBlackboard::with_parent("level3", level2.clone(), remappings3, true);

	let old = level0
		.set("levelA", String::from("testA"))
		.unwrap();
	assert_eq!(old, None);

	let old = level0
		.set("level0", String::from("test0"))
		.unwrap();
	assert_eq!(old, None);
	let old = level1
		.set("level1", String::from("test1"))
		.unwrap();
	assert_eq!(old, None);
	let old = level2
		.set("level2", String::from("test2"))
		.unwrap();
	assert_eq!(old, None);
	let old = level3
		.set("level3", String::from("test3"))
		.unwrap();
	assert_eq!(old, None);

	// test autoremap
	let old = level3
		.set("level2", String::from("changed2"))
		.unwrap();
	assert_eq!(old, Some(String::from("test2")));
	let value = level3.get::<String>("level2").unwrap();
	assert_eq!(value, String::from("changed2"));
	let value = level3.get::<String>("level1").unwrap();
	assert_eq!(value, String::from("test1"));
	let value = level3.get::<String>("level0").unwrap();
	assert_eq!(value, String::from("test0"));
	let old = level3
		.set("level0", String::from("changed0"))
		.unwrap();
	assert_eq!(old, Some(String::from("test0")));
	let value = level3.get::<String>("level0").unwrap();
	assert_eq!(value, String::from("changed0"));
	let old = level3.set("level0", 42);
	assert!(old.is_err());
	let _ = level3.delete::<String>("level0").unwrap();
	assert_eq!(value, String::from("changed0"));
	let old = level3.set("level0", 42).unwrap();
	assert_eq!(old, None);
	let value = level1.get::<i32>("level0").unwrap();
	assert_eq!(value, 42);

	// test manual remapping
	let value = level3.get::<String>("levelD").unwrap();
	assert_eq!(value, String::from("testA"));
	let value = level2.get::<String>("levelC").unwrap();
	assert_eq!(value, String::from("testA"));
	let old = level3
		.set("levelD", String::from("changedD"))
		.unwrap();
	assert_eq!(old, Some(String::from("testA")));
	let value = level1.get::<String>("levelB").unwrap();
	assert_eq!(value, String::from("changedD"));
	let old = level3.set("levelD", 42);
	assert!(old.is_err());
	let _ = level3.delete::<String>("levelD").unwrap();
	assert_eq!(value, String::from("changedD"));
	let old = level3.set("levelD", 42).unwrap();
	assert_eq!(old, None);
	let value = level1.get::<i32>("levelB").unwrap();
	assert_eq!(value, 42);
}
