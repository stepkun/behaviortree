#![no_main]
#![no_std]

//! Embedded version of [t03_generic_ports](examples/t03_generic_ports.rs)

use ariel_os::debug::{ExitCode, exit, log::*};

use behaviortree::prelude::*;

const XML: &str = r#"
<root BTCPP_format="4"
		main_tree_to_execute="MainTree">
	<BehaviorTree ID="MainTree">
		<Sequence name="root">
            <CalculateGoal   goal="{GoalPosition}" />
            <PrintTarget     target="{GoalPosition}" />
            <Script          code="OtherGoal:='-1;3'" />
            <PrintTarget     target="{OtherGoal}" />
		</Sequence>
	</BehaviorTree>
</root>
"#;

/// `Position2D`
#[derive(Clone, Debug, Default)]
pub struct Position2D {
    /// x value
    pub x: f64,
    /// y value
    pub y: f64,
}

impl FromStr for Position2D {
    type Err = core::num::ParseFloatError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        info!("Converting string: \"{}\"", value);
        // remove redundant ' and &apos; from string
        let s = value
            .replace('\'', "")
            .trim()
            .replace("&apos;", "")
            .trim()
            .to_string();
        let v: Vec<&str> = s.split(';').collect();
        let x = f64::from_str(v[0])?;
        let y = f64::from_str(v[1])?;
        Ok(Self { x, y })
    }
}

impl alloc::fmt::Display for Position2D {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

/// Action `CalculateGoal`
#[derive(Action, Debug, Default)]
pub struct CalculateGoal {}

#[async_trait::async_trait]
impl BehaviorInstance for CalculateGoal {
    async fn tick(
        &mut self,
        behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> BehaviorResult {
        let mygoal = Position2D { x: 1.1, y: 2.3 };
        behavior.set("goal", mygoal)?;
        Ok(BehaviorState::Success)
    }
}

impl BehaviorStatic for CalculateGoal {
    fn provided_ports() -> PortList {
        port_list![output_port!(Position2D, "goal")]
    }
}

/// Action `PrintTarget`
#[derive(Action, Debug, Default)]
pub struct PrintTarget {}

#[async_trait::async_trait]
impl BehaviorInstance for PrintTarget {
    async fn tick(
        &mut self,
        behavior: &mut BehaviorData,
        _children: &mut ConstBehaviorTreeElementList,
        _runtime: &SharedRuntime,
    ) -> BehaviorResult {
        let pos = behavior.get::<Position2D>("target")?;
        info!("Target positions: [ {}, {} ]", pos.x, pos.y);
        Ok(BehaviorState::Success)
    }
}

impl BehaviorStatic for PrintTarget {
    fn provided_ports() -> PortList {
        port_list![input_port!(Position2D, "target")]
    }
}

async fn example() -> BehaviorTreeResult {
    let mut factory = BehaviorTreeFactory::with_core_behaviors()?;

    register_behavior!(factory, CalculateGoal, "CalculateGoal")?;
    register_behavior!(factory, PrintTarget, "PrintTarget")?;

    let mut tree = factory.create_from_text(XML)?;
    // dropping the factory to free memory
    drop(factory);

    let result = tree.tick_while_running().await?;
    Ok(result)
}

#[ariel_os::task(autostart)]
async fn main() {
    info!("running t03_generic_ports...");
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
