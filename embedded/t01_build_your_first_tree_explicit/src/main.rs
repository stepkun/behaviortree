#![no_main]
#![no_std]

//! Embedded version of [t01_buid_your_first_tree_explicit](examples/t01_build_your_first_tree_explicit.rs)

use ariel_os::debug::{ExitCode, exit, log::*};

use behaviortree::prelude::*;

const XML: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Control ID="Sequence" name="root_sequence">
			<Condition ID="CheckBattery"	name="battery_ok"/>
			<Action ID="OpenGripper"		name="open_gripper"/>
			<Action ID="ApproachObject"		name="approach_object"/>
			<Action ID="CloseGripper"		name="close_gripper"/>
		</Control>
	</BehaviorTree>
</root>
"#;

/// Function for condition `CheckBattery`
/// # Errors
/// In this case never
pub fn check_battery() -> BehaviorResult {
    info!("[ Battery: OK ]");
    Ok(BehaviorState::Success)
}

/// Action `ApproachObject`
/// Example of custom `ActionNode` (synchronous action) without ports.
#[derive(Action, Debug, Default)]
pub struct ApproachObject {}

#[async_trait::async_trait]
impl BehaviorInstance for ApproachObject {
    async fn tick(
        &mut self,
        _behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> BehaviorResult {
        info!("ApproachObject: approach_object");
        Ok(BehaviorState::Success)
    }
}

impl BehaviorStatic for ApproachObject {}

/// Struct for behaviors `OpenGripper` and `CloseGripper`
#[derive(Default)]
pub struct GripperInterface {
    open: bool,
}

impl GripperInterface {
    /// Open the gripper.
    /// # Errors
    /// In this case never
    pub fn open(&mut self) -> BehaviorResult {
        info!("GripperInterface::open");
        self.open = true;
        Ok(BehaviorState::Success)
    }
    /// Close the gripper.
    /// # Errors
    /// In this case never
    pub fn close(&mut self) -> BehaviorResult {
        info!("GripperInterface::close");
        self.open = false;
        Ok(BehaviorState::Success)
    }
}

async fn example() -> BehaviorTreeResult {
    let mut factory = BehaviorTreeFactory::default();

    // The recommended way to create a Behavior is through inheritance/composition.
    // Even if it requires more boilerplate, it allows you to use more functionalities
    // like ports (we will discuss this in future tutorials).
    register_behavior!(factory, ApproachObject, "ApproachObject")?;

    // Registering a SimpleAction/SimpleCondition using a function pointer.
    register_behavior!(factory, check_battery, "CheckBattery", BehaviorKind::Condition)?;

    // You can also create SimpleAction/SimpleCondition using methods of a struct.
    register_behavior!(factory, GripperInterface::default(),
        open, "OpenGripper", BehaviorKind::Action,
        close, "CloseGripper", BehaviorKind::Action)?;

    // Trees are created at run-time, but only once at the beginning.
    // The currently supported format is XML.
    // IMPORTANT: When the object "tree" goes out of scope, all the tree components are destroyed
    let mut tree = factory.create_from_text(XML)?;
    // dropping the factory to free memory
    drop(factory);

    // To "execute" a Tree you need to "tick" it.
    // The tick is propagated to the children based on the logic of the tree.
    // In this case, the entire sequence is executed, because all the children
    // of the Sequence return SUCCESS.
    let result = tree.tick_while_running().await?;
    Ok(result)
}

#[ariel_os::task(autostart)]
async fn main() {
    info!("running t01_build_your_first_tree_explicit...");
    match example().await {
        Ok(_) => {
            info!("...succeeded!");
            exit(ExitCode::SUCCESS)
        },
        Err(_) => {
            error!("...failed!");
            exit(ExitCode::FAILURE)
        },
    };
}
