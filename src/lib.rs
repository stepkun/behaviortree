// Copyright © 2024 Stephan Kunz
#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(doctest)]
doc_comment::doctest!("../README.md");

#[doc(hidden)]
extern crate alloc;

// mostly needed stuff
pub mod prelude;

// modules
pub mod behavior; // due to macros!!
mod blackboard;
mod error;
pub mod factory; // due to macros!!
pub mod port; // due to macros!!
mod tree;
mod xml;

// flatten:
pub use behavior::{Behavior, BehaviorExecution};
pub use behavior::{BehaviorData, BehaviorDescription, BehaviorError, BehaviorKind, BehaviorResult, BehaviorState};
pub use blackboard::{Blackboard, BlackboardData, BlackboardInterface, SharedBlackboard};
pub use error::{BehaviorTreeResult, Error};
pub use factory::BehaviorTreeFactory;
pub use port::PortList;
#[cfg(feature = "std")]
pub use tree::observer::groot2_connector::Groot2Connector;
#[cfg(feature = "std")]
pub use tree::observer::tree_observer::BehaviorTreeObserver;
pub use tree::{BehaviorTree, BehaviorTreeElement, BehaviorTreeElementList};
pub use xml::creator::XmlCreator;

// re-exports:
pub use behaviortree_derive::{Action, Condition, Control, Decorator};

// region:		--- modules
use alloc::sync::Arc;
// endregion:	--- modules

// region		--- types
/// An immutable thread safe `String` type
/// see: [Logan Smith](https://www.youtube.com/watch?v=A4cKi7PTJSs).
pub type ConstString = Arc<str>;
// endregion:   --- types

// region:		--- globals
/// Often needed empty str
pub const EMPTY_STR: &str = "";

/// [`BehaviorState`] literal "Failure"
pub const FAILURE: &str = "Failure";
/// [`BehaviorState`] literal "Idle"
pub const IDLE: &str = "Idle";
/// [`BehaviorState`] literal "Running"
pub const RUNNING: &str = "Running";
/// [`BehaviorState`] literal "Skipped"
pub const SKIPPED: &str = "Skipped";
/// [`BehaviorState`] literal "Success"
pub const SUCCESS: &str = "Success";

/// [`BehaviorKind`] literal "Action"
pub const ACTION: &str = "Action";
/// [`BehaviorKind`] literal "Condition"
pub const CONDITION: &str = "Condition";
/// [`BehaviorKind`] literal "Control"
pub const CONTROL: &str = "Control";
/// [`BehaviorKind`] literal "Decorator"
pub const DECORATOR: &str = "Decorator";
/// [`BehaviorKind`] literal "Subtree"
pub const SUBTREE: &str = "SubTree";

/// Literal "name" for ports etc.
const NAME: &str = "name";
/// Literal "ID" for ports etc.
const ID: &str = "ID";
/// Literals for scripting ports
const AUTOREMAP: &str = "_autoremap";
const FAILURE_IF: &str = "_failureIf";
const SUCCESS_IF: &str = "_successIf";
const SKIP_IF: &str = "_skipIf";
const WHILE: &str = "_while";
const ON_HALTED: &str = "_onHalted";
const ON_FAILURE: &str = "_onFailure";
const ON_SUCCESS: &str = "_onSuccess";
const POST: &str = "_post";
// endregion:	--- globals

// region:		--- helpers
/// Removes enclosing brackets `{}` from a str if there are any,
/// otherwise returns the unchanged str.
#[must_use]
pub fn strip_curly_brackets(key: &str) -> &str {
	let key = key.strip_prefix('{').unwrap_or(key);
	key.strip_suffix('}').unwrap_or(key)
}
// endregion:	--- helpers

// region:		---macros
/// Macro to register different kinds of behaviors.
///
/// # Usage:
///
/// Register a Behavior (may be generic):
/// ```no-test
/// register_behavior!(<mutable (reference to) behavior factory>, <behavior to register>, <"identifying name">)
/// ```
///
/// Register a Behavior with additional arguments for construction:
/// ```no-test
/// register_behavior!(<mutable (reference to) behavior factory>, <behavior to register>, <"identifying name">, <arg1>, <arg2>, ...)
/// ```
///
/// Register a simple function as Behavior:
/// ```no-test
/// register_behavior!(<mutable (reference to) behavior factory>, <function to register>, <"identifying name">, BehaviorKind::<kind>)
/// ```
///
/// Register a simple function with ports as Behavior:
/// ```no-test
/// let some_ports = port_list! {input_port!(<port type, <port name>)};
/// register_behavior!(<mutable (reference to) behavior factory>, <function to register>, <"identifying name">, some_ports, BehaviorKind::<kind>)
/// ```
///
/// Register a struct with multiple functions:
/// ```no-test
/// let wrapped_struct = register_behavior!(factory, <struct_item>,
///         <first_func>, "NameForFirstFunc", BehaviorKind::<kind of first func>,
///         <second_func>, "NameForSecondFunc", BehaviorKind::<kind of second func>,
///         ...
/// )?;
/// /// ```
///
/// # Example:
///
/// ```no-test
/// let mut factory = BehaviorTreeFactory::with_core_behaviors()?;
/// // register derived behaviors:
/// register_behavior!(factory, ActionA, "Action_A")?;
/// register_behavior!(factory, ActionB, "Action_B", 42, "hello world".into())?;
/// register_behavior!(factory, Loop<Pose2D>, "LoopPose")?;
/// ```
#[macro_export]
macro_rules! register_behavior {
	// function
	($factory:expr, $fn:path, $name:literal, $kind:path $(,)?) => {{
		$factory.register_simple_function($name, alloc::sync::Arc::new($fn), $kind)
	}};
	// function with ports
	($factory:expr, $fn:path, $name:literal, $ports:expr, $kind:path $(,)?) => {{
		$factory.register_simple_function_with_ports($name, alloc::sync::Arc::new($fn), $kind, $ports)
	}};
	// behavior type struct
	($factory:expr, $tp:ty, $name:literal $(,)?) => {{
		$factory.register_behavior_type::<$tp>($name)
	}};
	// behavior type struct with arguments for construction
	($factory:expr, $tp:ty, $name:literal, $($arg:expr),* $(,)?) => {{
		let bhvr_desc = $crate::behavior::BehaviorDescription::new($name, stringify!($tp), <$tp>::kind(), false, <$tp>::provided_ports());
		let bhvr_creation_fn = alloc::boxed::Box::new(move || -> alloc::boxed::Box<dyn $crate::behavior::BehaviorExecution> {
			alloc::boxed::Box::new(<$tp>::new($($arg),*))
		});
		$factory
			.registry_mut()
			.add_behavior(bhvr_desc, bhvr_creation_fn)
	}};
	// multiple methods of a struct - will indicate only the last error if any
	// this needs to be last becaus the second argument beiing an expression covers most other kinds!!
	// returns an Arc-Mutex-wrapped item of the given struct
	($factory:expr, $item:expr, $($fun:ident, $name:literal, $kind:path $(,)?)+) => {{
		let base = alloc::sync::Arc::new(spin::Mutex::new($item));
		// let mut res: core::result::Result<alloc::sync::Arc<spin::Mutex<$item>>, $crate::factory::error::Error> = Ok(base.clone());
		let mut res = Ok(base.clone());
		$({
			let item = base.clone();
			if let Err(err) =$factory.register_simple_function($name, alloc::sync::Arc::new(move || { item.lock().$fun() }), $kind) {
				res = Err(err);
			}
		})+;
		res
	}};
}

/// Macro to register enums for scripting.
/// Enum must derive [`ScriptEnum`](https://docs.rs/tinyscript/latest/tinyscript/trait.ScriptEnum.html).
/// It is also possible to register discrete value(s).
///
/// # Usage:
///
/// With an enum type:
/// ```no-test
/// register_scripting_enum!(<mutable reference to behavior factory>, <enum to register>)
/// ```
///
/// With discrete value(s)
/// ```no-test
/// register_scripting_enum!(<mutable reference to behavior factory>, <Identifier as str>, <Value as int>)
/// ```
///
/// # Examples:
///
/// ```no-test
/// #[derive(ScriptEnum)]
/// enum Color {
///     RED,
///     BLUE,
///     GREEN,
/// }
///
/// register_scripting_enum!(factory, Color);
/// ```
///
/// ```no-test
/// register_scripting_enum!(factory "THE_ANSWER", 42, "OTHER_ANSWER", 44);
/// ```
#[macro_export]
macro_rules! register_scripting_enum {
	// register an enum type
	($factory:ident, $tp:ty) => {
		for (key, value) in <$tp>::key_value_tuples() {
			$factory.register_enum_tuple(key, value)?;
		}
	};
	// register a key value pair
	($factory:ident, $($key:literal, $value:literal),+ $(,)?) => {
		$( $factory.register_enum_tuple($key, $value)?; )+;
	};
}
// endregion:	---macros
