# behaviortree

'behaviortree' implements a behavior tree library similar to [BehaviorTree.CPP](https://www.behaviortree.dev/) but in `Rust`.

Examples implementing the BehaviorTree.CPP [tutorials](https://www.behaviortree.dev/docs/intro)
can be found [here](https://github.com/stepkun/behaviortree/tree/main/examples).
For __embedded__ devices similar examples are available [here](https://github.com/stepkun/behaviortree/tree/main/embedded)

⚠️ WARNING ⚠️
This crate is still in development, but will follow semantic versioning.

## Example

Below is a very simple example using functions as `Actions`.

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

## License

Licensed with the fair use "NGMC" license, see [license file](https://github.com/stepkun/behaviortree/blob/main/LICENSE)

## Contribution

Any contribution intentionally submitted for inclusion in the work by you,
shall be licensed with the same "NGMC" license, without any additional terms or conditions.
