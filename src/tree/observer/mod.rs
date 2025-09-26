// Copyright © 2025 Stephan Kunz

//! [`behaviortree`](crate) tree observer module.

#[cfg(feature = "std")]
pub mod groot2_connector;
#[cfg(feature = "std")]
pub mod groot2_protocol;
#[cfg(feature = "std")]
pub mod tree_observer;

// flatten

#[cfg(test)]
mod tests {
	use crate::{
		BehaviorTreeObserver, Groot2Connector,
		tree::observer::{
			groot2_connector::Groot2ConnectorData,
			groot2_protocol::{Groot2Hook, Groot2ReplyHeader, Groot2RequestHeader, Groot2RequestType},
			tree_observer::Statistics,
		},
	};

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Statistics>();
		is_normal::<BehaviorTreeObserver>();
		is_normal::<Groot2Connector>();
		is_normal::<Groot2ConnectorData>();
		is_normal::<Groot2Hook>();
		is_normal::<Groot2ReplyHeader>();
		is_normal::<Groot2RequestHeader>();
		is_normal::<Groot2RequestType>();
	}
}
