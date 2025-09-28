// Copyright © 2025 Stephan Kunz
//! Embedded control tests.

#![no_main]
#![no_std]
#![allow(clippy::unwrap_used)]

extern crate alloc;

#[cfg(test)]
#[embedded_test::tests]
mod tests {
	use behaviortree::{behavior::action::ChangeStateAfter, prelude::*};

	const FALLBACK: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Fallback name="root_fallback">
			<Condition ID="Condition" name="condition"/>
			<Action	ID="Action" name="action"/>
		</Fallback>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn fallback() -> Result<(), Error> {
		fn set_values(tree: &mut BehaviorTree, condition_state: BehaviorState, action_state: BehaviorState) {
			for behavior in tree.iter_mut() {
				if behavior.name().as_ref() == "condition" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(condition_state);
					}
				}
				if behavior.name().as_ref() == "action" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action_state);
					}
				}
			}
		}

		let mut factory = BehaviorTreeFactory::new()?;
		register_behavior!(
			factory,
			ChangeStateAfter,
			"Condition",
			BehaviorState::Running,
			BehaviorState::Failure,
			0
		)?;
		register_behavior!(
			factory,
			ChangeStateAfter,
			"Action",
			BehaviorState::Running,
			BehaviorState::Failure,
			0
		)?;
		let mut tree = factory.create_from_text(FALLBACK)?;
		drop(factory);

		// case 1
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 2
		set_values(&mut tree, BehaviorState::Success, BehaviorState::Failure);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		// case 3
		set_values(&mut tree, BehaviorState::Failure, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		// case 4
		set_values(&mut tree, BehaviorState::Success, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);

		Ok(())
	}

	const IF_THEN_ELSE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<IfThenElse name="root_if_then_else">
			<Condition ID="Condition" name="if"/>
			<Action ID="Action" name="then"/>
			<Action	ID="Action" name="else"/>
		</IfThenElse>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn if_then_else() -> Result<(), Error> {
		fn set_values(
			tree: &mut BehaviorTree,
			condition_state: BehaviorState,
			then_action_state: BehaviorState,
			else_action_state: BehaviorState,
		) {
			for behavior in tree.iter_mut() {
				if behavior.name().as_ref() == "if" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(condition_state);
					}
				}
				if behavior.name().as_ref() == "then" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(then_action_state);
					}
				}
				if behavior.name().as_ref() == "else" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(else_action_state);
					}
				}
			}
		}

		let mut factory = BehaviorTreeFactory::with_extended_behaviors()?;
		register_behavior!(
			factory,
			ChangeStateAfter,
			"Condition",
			BehaviorState::Running,
			BehaviorState::Failure,
			0
		)?;
		register_behavior!(
			factory,
			ChangeStateAfter,
			"Action",
			BehaviorState::Running,
			BehaviorState::Failure,
			0
		)?;
		let mut tree = factory.create_from_text(IF_THEN_ELSE)?;
		drop(factory);

		// case 1
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 2
		set_values(&mut tree, BehaviorState::Success, BehaviorState::Failure, BehaviorState::Idle);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 3
		set_values(&mut tree, BehaviorState::Success, BehaviorState::Success, BehaviorState::Idle);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		// case 4
		set_values(&mut tree, BehaviorState::Failure, BehaviorState::Idle, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		// case 4
		set_values(&mut tree, BehaviorState::Success, BehaviorState::Success, BehaviorState::Idle);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		set_values(&mut tree, BehaviorState::Failure, BehaviorState::Idle, BehaviorState::Failure);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);

		Ok(())
	}

	const PARALLEL_ALL: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<ParallelAll name="root_parallel_all" max_failures="1">
			<Action ID="Action" name="action1"/>
			<Action ID="Action" name="action2"/>
			<Action	ID="Action" name="action3"/>
		</ParallelAll>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn parallel_all() -> Result<(), Error> {
		fn set_values(
			tree: &mut BehaviorTree,
			action1_state: BehaviorState,
			action2_state: BehaviorState,
			action3_state: BehaviorState,
		) {
			for behavior in tree.iter_mut() {
				if behavior.name().as_ref() == "action1" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action1_state);
					}
				}
				if behavior.name().as_ref() == "action2" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action2_state);
					}
				}
				if behavior.name().as_ref() == "action3" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action3_state);
					}
				}
			}
		}

		let mut factory = BehaviorTreeFactory::with_core_behaviors()?;
		register_behavior!(
			factory,
			ChangeStateAfter,
			"Action",
			BehaviorState::Running,
			BehaviorState::Failure,
			0
		)?;
		let mut tree = factory.create_from_text(PARALLEL_ALL)?;
		drop(factory);

		// case 1
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 2
		tree.reset()?;
		set_values(
			&mut tree,
			BehaviorState::Success,
			BehaviorState::Failure,
			BehaviorState::Failure,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 3
		tree.reset()?;
		set_values(
			&mut tree,
			BehaviorState::Success,
			BehaviorState::Success,
			BehaviorState::Success,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		// case 4
		tree.reset()?;
		set_values(
			&mut tree,
			BehaviorState::Failure,
			BehaviorState::Success,
			BehaviorState::Success,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);

		Ok(())
	}

	const PARALLEL: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Parallel name="root_parallel" success_count="1" failure_count="1">
			<Action ID="Action" name="action1"/>
			<Action ID="Action" name="action2"/>
			<Action	ID="Action" name="action3"/>
		</Parallel>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn parallel() -> Result<(), Error> {
		fn set_values(
			tree: &mut BehaviorTree,
			action1_state: BehaviorState,
			action2_state: BehaviorState,
			action3_state: BehaviorState,
		) {
			for behavior in tree.iter_mut() {
				if behavior.name().as_ref() == "action1" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action1_state);
					}
				}
				if behavior.name().as_ref() == "action2" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action2_state);
					}
				}
				if behavior.name().as_ref() == "action3" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action3_state);
					}
				}
			}
		}

		let mut factory = BehaviorTreeFactory::with_core_behaviors()?;
		register_behavior!(
			factory,
			ChangeStateAfter,
			"Action",
			BehaviorState::Running,
			BehaviorState::Failure,
			0
		)?;
		let mut tree = factory.create_from_text(PARALLEL)?;
		drop(factory);

		// case 1
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 2
		tree.reset()?;
		set_values(
			&mut tree,
			BehaviorState::Success,
			BehaviorState::Failure,
			BehaviorState::Failure,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 3
		tree.reset()?;
		set_values(
			&mut tree,
			BehaviorState::Success,
			BehaviorState::Success,
			BehaviorState::Success,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		// case 4
		tree.reset()?;
		set_values(
			&mut tree,
			BehaviorState::Failure,
			BehaviorState::Success,
			BehaviorState::Success,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);

		Ok(())
	}

	const REACTIVE_FALLBACK: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<ReactiveFallback name="root_fallback">
			<Condition ID="Condition" name="condition1"/>
			<Condition ID="Condition" name="condition2"/>
			<Action	ID="Action" name="action"/>
		</ReactiveFallback>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn reactive_fallback() -> Result<(), Error> {
		fn set_values(
			tree: &mut BehaviorTree,
			condition1_state: BehaviorState,
			condition2_state: BehaviorState,
			action_state: BehaviorState,
		) {
			for behavior in tree.iter_mut() {
				if behavior.name().as_ref() == "condition1" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(condition1_state);
					}
				}
				if behavior.name().as_ref() == "condition2" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(condition2_state);
					}
				}
				if behavior.name().as_ref() == "action" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action_state);
					}
				}
			}
		}

		let mut factory = BehaviorTreeFactory::with_core_behaviors()?;
		register_behavior!(
			factory,
			ChangeStateAfter,
			"Condition",
			BehaviorState::Running,
			BehaviorState::Failure,
			0
		)?;
		register_behavior!(
			factory,
			ChangeStateAfter,
			"Action",
			BehaviorState::Running,
			BehaviorState::Failure,
			0
		)?;
		let mut tree = factory.create_from_text(REACTIVE_FALLBACK)?;
		drop(factory);

		// case 1
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 2
		set_values(
			&mut tree,
			BehaviorState::Failure,
			BehaviorState::Failure,
			BehaviorState::Success,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		// case 3
		set_values(
			&mut tree,
			BehaviorState::Failure,
			BehaviorState::Failure,
			BehaviorState::Failure,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		set_values(
			&mut tree,
			BehaviorState::Success,
			BehaviorState::Failure,
			BehaviorState::Failure,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);

		Ok(())
	}

	const REACTIVE_SEQUENCE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<ReactiveSequence name="root_sequence">
			<Action ID="Action" name="action1"/>
			<Action ID="Action" name="action2"/>
			<Action	ID="Action" name="action3"/>
		</ReactiveSequence>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn reactive_sequence_raw() -> Result<(), Error> {
		fn set_values(
			tree: &mut BehaviorTree,
			action1_state: BehaviorState,
			action2_state: BehaviorState,
			action3_state: BehaviorState,
		) {
			for behavior in tree.iter_mut() {
				if behavior.name().as_ref() == "action1" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action1_state);
					}
				}
				if behavior.name().as_ref() == "action2" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action2_state);
					}
				}
				if behavior.name().as_ref() == "action3" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action3_state);
					}
				}
			}
		}

		let mut factory = BehaviorTreeFactory::with_core_behaviors()?;
		register_behavior!(
			factory,
			ChangeStateAfter,
			"Action",
			BehaviorState::Running,
			BehaviorState::Failure,
			0
		)?;
		let mut tree = factory.create_from_text(REACTIVE_SEQUENCE)?;
		drop(factory);

		// case 1
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 2
		set_values(
			&mut tree,
			BehaviorState::Success,
			BehaviorState::Failure,
			BehaviorState::Success,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 3
		set_values(
			&mut tree,
			BehaviorState::Success,
			BehaviorState::Success,
			BehaviorState::Success,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		set_values(
			&mut tree,
			BehaviorState::Failure,
			BehaviorState::Success,
			BehaviorState::Success,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);

		Ok(())
	}

	const SEQUENCE_WITH_MEMORY: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<SequenceWithMemory name="root_sequence">
			<Action ID="Action" name="action1"/>
			<Action ID="Action" name="action2"/>
			<Action	ID="Action" name="action3"/>
		</SequenceWithMemory>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn sequence_with_memory() -> Result<(), Error> {
		fn set_values(
			tree: &mut BehaviorTree,
			action1_state: BehaviorState,
			action2_state: BehaviorState,
			action3_state: BehaviorState,
		) {
			for behavior in tree.iter_mut() {
				if behavior.name().as_ref() == "action1" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action1_state);
					}
				}
				if behavior.name().as_ref() == "action2" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action2_state);
					}
				}
				if behavior.name().as_ref() == "action3" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action3_state);
					}
				}
			}
		}

		let mut factory = BehaviorTreeFactory::with_core_behaviors()?;
		register_behavior!(
			factory,
			ChangeStateAfter,
			"Action",
			BehaviorState::Running,
			BehaviorState::Failure,
			0
		)?;
		let mut tree = factory.create_from_text(SEQUENCE_WITH_MEMORY)?;
		drop(factory);

		// case 1
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 2
		set_values(
			&mut tree,
			BehaviorState::Success,
			BehaviorState::Failure,
			BehaviorState::Success,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 3
		set_values(
			&mut tree,
			BehaviorState::Success,
			BehaviorState::Success,
			BehaviorState::Running,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Running);
		set_values(
			&mut tree,
			BehaviorState::Failure,
			BehaviorState::Success,
			BehaviorState::Success,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);

		Ok(())
	}

	const SEQUENCE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Sequence name="root_sequence">
			<Action ID="Action" name="action1"/>
			<Action ID="Action" name="action2"/>
			<Action	ID="Action" name="action3"/>
		</Sequence>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn sequence() -> Result<(), Error> {
		fn set_values(
			tree: &mut BehaviorTree,
			action1_state: BehaviorState,
			action2_state: BehaviorState,
			action3_state: BehaviorState,
		) {
			for behavior in tree.iter_mut() {
				if behavior.name().as_ref() == "action1" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action1_state);
					}
				}
				if behavior.name().as_ref() == "action2" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action2_state);
					}
				}
				if behavior.name().as_ref() == "action3" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action3_state);
					}
				}
			}
		}

		let mut factory = BehaviorTreeFactory::new()?;
		register_behavior!(
			factory,
			ChangeStateAfter,
			"Action",
			BehaviorState::Running,
			BehaviorState::Failure,
			0
		)?;
		let mut tree = factory.create_from_text(SEQUENCE)?;
		drop(factory);

		// case 1
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 2
		set_values(
			&mut tree,
			BehaviorState::Success,
			BehaviorState::Failure,
			BehaviorState::Success,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 3
		set_values(
			&mut tree,
			BehaviorState::Success,
			BehaviorState::Success,
			BehaviorState::Success,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		set_values(
			&mut tree,
			BehaviorState::Failure,
			BehaviorState::Success,
			BehaviorState::Success,
		);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);

		Ok(())
	}

	const SWITCH: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Switch3 name="root_switch" variable="{var}"  case_1="1" case_2="2" case_3="3">
			<Action ID="Action" name="case1"/>
			<Action ID="Action" name="case2"/>
			<Action	ID="Action" name="case3"/>
			<Action	ID="Action" name="default"/>
		</Switch3>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn switch() -> Result<(), Error> {
		fn set_values(
			tree: &mut BehaviorTree,
			action1_state: BehaviorState,
			action2_state: BehaviorState,
			action3_state: BehaviorState,
			default_state: BehaviorState,
		) {
			for behavior in tree.iter_mut() {
				if behavior.name().as_ref() == "case1" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action1_state);
					}
				}
				if behavior.name().as_ref() == "case2" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action2_state);
					}
				}
				if behavior.name().as_ref() == "case3" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(action3_state);
					}
				}
				if behavior.name().as_ref() == "default" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(default_state);
					}
				}
			}
		}

		let mut factory = BehaviorTreeFactory::with_groot2_behaviors()?;
		register_behavior!(
			factory,
			ChangeStateAfter,
			"Action",
			BehaviorState::Running,
			BehaviorState::Failure,
			0
		)?;
		factory.register_behavior_tree_from_text(SWITCH)?;

		let blackboard = Databoard::new();
		let mut tree = factory.create_tree_with("MainTree", &blackboard)?;
		drop(factory);

		// preparation
		blackboard.set("var", String::from("1"))?;
		set_values(
			&mut tree,
			BehaviorState::Success,
			BehaviorState::Failure,
			BehaviorState::Running,
			BehaviorState::Skipped,
		);
		// case 1
		// blackboard.set("var", 1)?;
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		// case 2
		blackboard.set("var", String::from("2"))?;
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 3
		blackboard.set("var", String::from("3"))?;
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Running);
		// default
		blackboard.set("var", String::from("42"))?;
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Skipped);

		Ok(())
	}

	const WHILE_DO_ELSE: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<WhileDoElse name="root_if_then_else">
			<Condition ID="Condition" name="while"/>
			<Action ID="Action" name="then"/>
			<Action	ID="Action" name="else"/>
		</WhileDoElse>
	</BehaviorTree>
</root>
"#;

	#[test]
	async fn while_do_else() -> Result<(), Error> {
		fn set_values(
			tree: &mut BehaviorTree,
			condition_state: BehaviorState,
			then_action_state: BehaviorState,
			else_action_state: BehaviorState,
		) {
			for behavior in tree.iter_mut() {
				if behavior.name().as_ref() == "while" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(condition_state);
					}
				}
				if behavior.name().as_ref() == "then" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(then_action_state);
					}
				}
				if behavior.name().as_ref() == "else" {
					if let Some(behavior) = behavior
						.behavior_mut()
						.as_any_mut()
						.downcast_mut::<ChangeStateAfter>()
					{
						behavior.set_final_state(else_action_state);
					}
				}
			}
		}

		let mut factory = BehaviorTreeFactory::with_extended_behaviors()?;
		register_behavior!(
			factory,
			ChangeStateAfter,
			"Condition",
			BehaviorState::Running,
			BehaviorState::Failure,
			0
		)?;
		register_behavior!(
			factory,
			ChangeStateAfter,
			"Action",
			BehaviorState::Running,
			BehaviorState::Failure,
			0
		)?;
		let mut tree = factory.create_from_text(WHILE_DO_ELSE)?;
		drop(factory);

		// case 1
		let mut result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 2
		set_values(&mut tree, BehaviorState::Success, BehaviorState::Failure, BehaviorState::Idle);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);
		// case 3
		set_values(&mut tree, BehaviorState::Success, BehaviorState::Success, BehaviorState::Idle);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		// case 4
		set_values(&mut tree, BehaviorState::Failure, BehaviorState::Idle, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		// case 5
		set_values(&mut tree, BehaviorState::Success, BehaviorState::Success, BehaviorState::Idle);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Success);
		set_values(&mut tree, BehaviorState::Failure, BehaviorState::Idle, BehaviorState::Failure);
		result = tree.tick_once().await?;
		assert_eq!(result, BehaviorState::Failure);

		Ok(())
	}
}
