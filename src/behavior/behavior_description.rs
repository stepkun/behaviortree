// Copyright © 2025 Stephan Kunz
//! [`BehaviorDescription`] implementation.

use crate::{BehaviorKind, ConstString, EMPTY_STR, port::PortList};

/// Description of a Behavior, used in xml parsing and creating.
#[derive(Clone, Debug, Default)]
pub struct BehaviorDescription {
	/// Name of the behavior, with which it is used in the [`BehaviorTree`](crate::tree::tree::BehaviorTree).
	name: ConstString,
	/// Id of the behavior under which it can be found in the [`BehaviorTreeFactory`](crate::factory::BehaviorTreeFactory).
	id: ConstString,
	/// Path to the element.
	/// In contrast to BehaviorTree.CPP this path is fully qualified,
	/// which means that every level is denoted explicitly, including the tree root.
	path: ConstString,
	/// Kind of the behavior.
	kind: BehaviorKind,
	/// The [`PortList`]
	ports: PortList,
	/// Flag to indicate whether this behavior is builtin by Groot2.
	groot2: bool,
	/// Path for Groot2
	groot2_path: ConstString,
}

impl BehaviorDescription {
	/// Create a behavior description.
	#[must_use]
	pub fn new(name: &str, id: &str, kind: BehaviorKind, groot2: bool, ports: PortList) -> Self {
		Self {
			name: name.into(),
			id: id.into(),
			path: EMPTY_STR.into(),
			kind,
			ports,
			groot2_path: EMPTY_STR.into(),
			groot2,
		}
	}

	/// Get name
	#[must_use]
	pub const fn name(&self) -> &ConstString {
		&self.name
	}

	/// Method to set the name.
	pub fn set_name(&mut self, name: &str) {
		self.name = name.into();
	}

	/// Get id
	#[must_use]
	pub const fn id(&self) -> &ConstString {
		&self.id
	}

	/// Method to get the path.
	#[must_use]
	pub const fn path(&self) -> &ConstString {
		&self.path
	}

	/// Method to set the path.
	pub fn set_path(&mut self, path: &str) {
		self.path = path.into();
	}

	/// Get kind
	#[must_use]
	pub const fn kind(&self) -> BehaviorKind {
		self.kind
	}

	/// Get kind as str
	#[must_use]
	pub const fn kind_str(&self) -> &'static str {
		self.kind.as_str()
	}

	/// Get ports
	#[must_use]
	pub const fn ports(&self) -> &PortList {
		&self.ports
	}

	/// If is builtin of Groot2
	#[must_use]
	pub const fn groot2(&self) -> bool {
		self.groot2
	}

	/// Get the path for Groot2.
	#[must_use]
	pub const fn groot2_path(&self) -> &ConstString {
		&self.groot2_path
	}

	/// Set the path for Groot2.
	pub fn set_groot2_path(&mut self, groot2_path: ConstString) {
		self.groot2_path = groot2_path;
	}
}
