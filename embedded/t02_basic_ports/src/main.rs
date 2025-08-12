#![no_main]
#![no_std]

//! Embedded version of [t02_basic_ports](examples/t02_basic_ports.rs)

use ariel_os::debug::{ExitCode, exit, log::*};

use behaviortree::prelude::*;

const XML: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Sequence name="root">
			<SaySomething     message="hello" />
			<SaySomething2    message="this works too" />
			<ThinkWhatToSay   text="{the_answer}"/>
			<SaySomething     message="{the_answer}" />
			<SaySomething2    message="{the_answer}" />
		</Sequence>
	</BehaviorTree>
</root>
"#;

/// Action `SaySomething`
/// Example of custom `ActionNode` (synchronous action) with an input port.
#[derive(Action, Debug, Default)]
pub struct SaySomething {}

#[async_trait::async_trait]
impl BehaviorInstance for SaySomething {
    async fn tick(
        &mut self,
        behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> BehaviorResult {
        let msg = behavior.get::<String>("message")?;
        info!("Robot says: {}", msg.as_str());
        Ok(BehaviorState::Success)
    }
}

impl BehaviorStatic for SaySomething {
    fn provided_ports() -> PortList {
        port_list! {input_port!(String, "message")}
    }
}

/// Same as struct `SaySomething`, but to be registered with `SimpleBehavior`
/// # Errors
#[allow(clippy::needless_pass_by_ref_mut)]
pub fn say_something_simple(behavior: &mut BehaviorData) -> BehaviorResult {
    let msg = behavior.get::<String>("message")?;
    info!("Robot2 says: {}", msg.as_str());
    Ok(BehaviorState::Success)
}

/// Action `ThinkWhatToSay`
#[derive(Action, Debug, Default)]
pub struct ThinkWhatToSay {}

#[async_trait::async_trait]
impl BehaviorInstance for ThinkWhatToSay {
    async fn tick(
        &mut self,
        behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> BehaviorResult {
        behavior.set("text", String::from("The answer is 42"))?;
        Ok(BehaviorState::Success)
    }
}

impl BehaviorStatic for ThinkWhatToSay {
    fn provided_ports() -> PortList {
        port_list![output_port!(String, "text")]
    }
}

async fn example() -> BehaviorTreeResult {
    let mut factory = BehaviorTreeFactory::default();

    // The struct SaySomething has a method called ports() that defines the INPUTS.
    // In this case, it requires an input called "message"
    register_behavior!(factory, SaySomething, "SaySomething")?;

    // Similarly to SaySomething, ThinkWhatToSay has an OUTPUT port called "text"
    // Both these ports are of type `String`, therefore they can connect to each other
    register_behavior!(factory, ThinkWhatToSay, "ThinkWhatToSay")?;

    // `SimpleBehavior` can not define their own method provided_ports(), therefore
    // we have to pass the PortsList explicitly if we want the Action to use get_input()
    // or set_output();
    let say_something_ports = port_list! {input_port!(String, "message")};
    register_behavior!(factory, say_something_simple, "SaySomething2", say_something_ports, BehaviorKind::Action)?;

    let mut tree = factory.create_from_text(XML)?;
    // dropping the factory to free memory
    drop(factory);

    let result = tree.tick_while_running().await?;
    Ok(result)
}

#[ariel_os::task(autostart)]
async fn main() {
    info!("running t02_basic_ports...");
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
