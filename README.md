# behaviortree

'behaviortree' implements a behavior tree library similar to [BehaviorTree.CPP](https://www.behaviortree.dev/) but in `Rust`.

Examples implementing the BehaviorTree.CPP [tutorials](https://www.behaviortree.dev/docs/intro)
can be found [here](https://github.com/stepkun/behaviortree/tree/main/examples).
For __embedded__ devices similar examples are available [here](https://github.com/stepkun/behaviortree/tree/main/embedded)

⚠️ WARNING ⚠️
This crate is still in development.

## Usage

Below is a very simple example using functions as `Actions`.
For more examples see: 
- [Linux/Mac-OS/Windows](https://github.com/stepkun/behaviortree/tree/main/examples), 
- [embedded](https://github.com/stepkun/behaviortree/tree/main/embedded)

```rust
use behaviortree::prelude::*;

const XML: &str = r#"
<root BTCPP_format="4">
    <BehaviorTree ID="MyBehavior">
        <Sequence>
			<MyAction1/>
			<MyAction2/>
        </Sequence>
    </BehaviorTree>
</root>
"#;

fn action_1() -> BehaviorResult {
    // your activity
    // ...

    // In case of Failure    
    //return Ok(BehaviorState::Failure);

    // In case of Success    
    Ok(BehaviorState::Success)
}

fn action_2() -> BehaviorResult {
    // your activity
    // ...
    Ok(BehaviorState::Success)
}

#[tokio::main]
async fn main() {
    // create a behavior factory
    let mut factory = BehaviorTreeFactory::default();

    // register your behaviors
    register_behavior!(factory, action_1, "MyAction1", BehaviorKind::Action).unwrap();
    register_behavior!(factory, action_2, "MyAction2", BehaviorKind::Action).unwrap();

    // create the tree
    let mut tree = factory.create_from_text(XML).unwrap();
    
    // run the tree until Success or Failure
    tree.tick_while_running().await.unwrap();
}
```

For implementation of your own complex behaviors, there is a set of 
derive macros: `Action`, `Condition`, `Control` and `Decorator`.

```rust
use behaviortree::prelude::*;

/// Derive an `Action`
#[derive(Action, Debug, Default)]
pub struct SaySomething {}

/// Implement the `Action`s functionality
#[async_trait::async_trait]
impl Behavior for SaySomething {
    /// Minimum implement the tick function
	async fn tick(
		&mut self,
		behavior: &mut BehaviorData,
		_children: &mut ConstBehaviorTreeElementList,
		_runtime: &SharedRuntime,
	) -> BehaviorResult {
        // getting the port
		let msg = behavior.get::<String>("message")?;
        // doing something
		println!("Robot says: {msg}");
        // signaling success
		Ok(BehaviorState::Success)
	}

    /// Define the available ports.
    /// This is optional, the default implementation is for no ports.
	fn provided_ports() -> PortList {
		port_list! {input_port!(String, "message")}
	}
}
```

## Capabilities

 ✅: Supported<br>
 🚦: Not yet fully tested but should work<br>
 🔴: Not yet supported<br>
 ??: Unclear if it can be supported<br>
 ❌: Will not be supported

### General capabilities

| Capability              | With OS | Embedded |
| ----------------------- | ------- | -------- |
| XML                     |         |          |
| - parsing               | ✅      | ✅       |
| - generation            | ✅      | ✅       |
|                         |         |          |
| Ports                   |         |          |
| - remapping             | ✅      | ✅       |
| - access by ref         | ✅      | ✅       |
|                         |         |          |
| Subtrees                |         |          |
| - structure             | ✅      | ✅       |
| - remapping             | ✅      | ✅       |
| - 'include' from file   | ✅      | ❌       |
|                         |         |          |
| Blackboard              |         |          |
| - hierarchy             | ✅      | ✅       |
| - remapping             | ✅      | ✅       |
| - access by ref         | ✅      | ✅       |
| - backup                | 🔴      | ??       |
|                         |         |          |
| Pre-/post-conditions    | ✅      | ✅       |
| Scripting               | ✅      | ✅       |
|                         |         |          |
| Loggers/Observers       | ✅      | ??       |
| Substitution rules      | 🔴      | ??       |
|                         |         |          |
| Using Groot2 for:       |         |          |
| - XML Create/Edit       | ✅      | ✅       |
| - Live Monitoring       | ✅      | ??       |
| - Pro Features          | 🔴      | ??       |

### Built-in behaviors

| BehaviorTree.CPP nodes    | With OS | Embedded |
| ------------------------- | ------- | -------- |
| __Action__                |         |          |
| `AlwaysFailure`           | ✅      | ✅       |
| `AlwaysSuccess`           | ✅      | ✅       |
| `Script`                  | ✅      | ✅       |
| `SetBlackboard`           | ✅      | 🚦       |
| `Sleep`                   | 🚦      | 🔴       |
| `UnsetBlackboard`         | ✅      | 🚦       |
| `PopFromQueue` (new)      | ✅      | 🚦       |
|                           |         |          |
| __Condition__             |         |          |
| `ScriptCondition`         | 🚦      | 🚦       |
| `WasEntryUpdated`         | ✅      | 🚦       |
|                           |         |          |
| __Control__               |         |          |
| `Fallback`                | ✅      | ✅       |
| `ReactiveFallback`        | ✅      | 🚦       |
| `Sequence`                | ✅      | ✅       |
| `ReactiveSequence`        | ✅      | ✅       |
| `SequenceWithMemory`      | ✅      | 🚦       |
| `Parallel`                | ✅      | 🚦       |
| `ParallelAll`             | ✅      | 🚦       |
| `IfThenElse`              | ✅      | 🚦       |
| `WhileDoElse`             | ✅      | 🚦       |
| `Switch`                  | ✅      | 🚦       |
| `ManualSelector` (new)    | 🔴      | ??       |
|                           |         |          |
| __Decorator__             |         |          |
| `ForceFailure`            | ✅      | 🚦       |
| `ForceSuccess`            | ✅      | 🚦       |
| `Inverter`                | ✅      | ✅       |
| `KeepRunningUntilFailure` | ✅      | 🚦       |
| `Repeat`                  | ✅      | 🚦       |
| `RetryUntilSuccessful`    | ✅      | 🚦       |
| `Delay`                   | 🚦      | 🔴       |
| `EntryUpdated`            | ✅      | 🚦       |
| `LoopQueue`               | ✅      | ✅       |
| `RunOnce`                 | ✅      | 🚦       |
| `ScriptPrecondition`      | 🚦      | 🚦       |
| `Timeout`                 | 🚦      | 🔴       |

## License

Licensed with the fair use "NGMC" license, see [license file](https://github.com/stepkun/behaviortree/blob/main/LICENSE)

## Contribution

Any contribution intentionally submitted for inclusion in the work by you,
shall be licensed with the same "NGMC" license, without any additional terms or conditions.
