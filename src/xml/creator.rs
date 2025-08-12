// Copyright © 2025 Stephan Kunz

//! XML writer for `behaviortree`

#[doc(hidden)]
#[cfg(feature = "std")]
extern crate std;

// region:      --- modules
use crate::{ConstString, SHOULD_NOT_HAPPEN};
use alloc::{
	collections::btree_map::BTreeMap,
	string::{String, ToString},
	vec::Vec,
};

use crate::{
	behavior::{
		BehaviorDescription,
		pre_post_conditions::{POST_CONDITIONS, PRE_CONDITIONS},
	},
	factory::BehaviorTreeFactory,
	tree::{tree::BehaviorTree, tree_element::BehaviorTreeElement, tree_element::TreeElementKind},
};
use woxml::XmlWriter;

// endregion:   --- modules

// region:      --- XmlWriter
/// Write different kinds of XML from various sources.
#[derive(Default)]
pub struct XmlCreator;

impl XmlCreator {
	/// Create XML `TreeNodesModel` from factories registered nodes.
	/// # Errors
	pub fn write_tree_nodes_model(factory: &BehaviorTreeFactory, pretty: bool) -> Result<ConstString, woxml::Error> {
		let mut writer = if pretty {
			XmlWriter::pretty_mode(Vec::new())
		} else {
			XmlWriter::compact_mode(Vec::new())
		};

		writer.begin_elem("root")?;
		writer.attr("BTCPP_format", "4")?;
		writer.begin_elem("TreeNodesModel")?;

		// loop over factories behavior entries in registry
		for item in factory.registry().behaviors() {
			if !item.1.0.groot2() {
				writer.begin_elem(item.1.0.kind_str())?;
				writer.attr("ID", item.0)?;
				// look for a PortsList
				for port in &item.1.0.ports().0 {
					writer.begin_elem(port.direction().type_str())?;
					writer.attr("name", port.name())?;
					writer.attr("type", port.type_name())?;
					writer.end_elem()?;
				}
				writer.end_elem()?;
			}
		}

		writer.end_elem()?; // TreeNodesModel
		writer.end_elem()?; // root
		writer.flush()?;
		let raw = writer.into_inner();
		let mut output = String::with_capacity(raw.len());
		for c in raw {
			output.push(c as char);
		}
		Ok(output.into())
	}

	/// Create XML from tree including `TreeNodesModel`.
	/// # Errors
	/// # Panics
	pub fn write_tree(
		tree: &BehaviorTree,
		metadata: bool,
		builtin_models: bool,
		pretty: bool,
	) -> Result<ConstString, woxml::Error> {
		// storage for (non groot2 builtin) behaviors to mention in TreeNodesModel
		let mut behaviors: BTreeMap<ConstString, BehaviorDescription> = BTreeMap::new();
		let mut subtrees: BTreeMap<ConstString, &BehaviorTreeElement> = BTreeMap::new();

		let mut writer = if pretty {
			XmlWriter::pretty_mode(Vec::new())
		} else {
			XmlWriter::compact_mode(Vec::new())
		};

		{
			writer.begin_elem("root")?;
			writer.attr("BTCPP_format", "4")?;

			// scan the tree
			for item in tree.iter() {
				#[allow(clippy::match_same_arms)]
				match item.kind() {
					TreeElementKind::Leaf => {
						let desc = item.data().description();
						if builtin_models || !desc.groot2() {
							behaviors.insert(desc.name().clone(), desc.clone());
						}
					}
					TreeElementKind::Node => {
						let desc = item.data().description();
						if builtin_models || !desc.groot2() {
							behaviors.insert(desc.name().clone(), desc.clone());
						}
					}
					TreeElementKind::SubTree => {
						subtrees.insert(item.data().description().path().clone(), item);
					}
				}
			}

			// create the BehaviorTree's
			for (_path, subtree) in subtrees {
				writer.begin_elem("BehaviorTree")?;
				writer.attr("ID", subtree.data().description().name())?;
				writer.attr("_fullpath", subtree.data().description().groot2_path())?;

				// recursive dive into children
				for element in subtree.children().iter() {
					Self::write_subtree(element, &mut writer, metadata)?;
				}
				writer.end_elem()?; // BehaviorTree
			}

			// create the TreeNodesModel
			writer.begin_elem("TreeNodesModel")?;
			// loop over collected behavior entries
			for (name, item) in &behaviors {
				if builtin_models || !item.groot2() {
					writer.begin_elem(item.kind_str())?;
					writer.attr("ID", name)?;
					// look for a PortsList
					for port in &item.ports().0 {
						writer.begin_elem(port.direction().type_str())?;
						writer.attr("name", port.name())?;
						writer.attr("type", port.type_name())?;
						if !port.description().is_empty() {
							writer.set_compact_mode();
							writer.text(port.description())?;
						}
						writer.end_elem()?;
						if pretty {
							writer.set_pretty_mode();
						}
					}
					writer.end_elem()?;
				}
			}

			writer.end_elem()?; // TreeNodesModel
			writer.end_elem()?; // root
			writer.flush()?;
		}

		let inner = writer.into_inner();
		let res = String::from_utf8(inner).expect(SHOULD_NOT_HAPPEN);
		Ok(res.into())
	}

	fn write_subtree<'a>(
		element: &'a BehaviorTreeElement,
		writer: &mut XmlWriter<'a, Vec<u8>>,
		metadata: bool,
	) -> Result<(), woxml::Error> {
		let is_subtree = match element.kind() {
			TreeElementKind::Leaf | TreeElementKind::Node => {
				writer.begin_elem(element.data().description().id())?;
				writer.attr("name", element.data().description().name())?;
				false
			}
			TreeElementKind::SubTree => {
				writer.begin_elem("SubTree")?;
				writer.attr("ID", element.data().description().name())?;
				if metadata {
					writer.attr("_fullpath", element.data().description().groot2_path())?;
				}
				true
			}
		};
		if metadata {
			writer.attr("_uid", &element.data().uid().to_string())?;
		}

		if is_subtree {
			// subtree port mappings/values are in blackboard
			if let Some(remappings) = element.data().blackboard().remappings() {
				for remapping in remappings.iter() {
					writer.attr(&remapping.0, &remapping.1)?;
				}
			}
		} else {
			// behavior port mappings/values
			for remapping in element.data().remappings().iter() {
				writer.attr(&remapping.0, &remapping.1)?;
			}
		}

		// Pre-conditions
		if let Some(conditions) = &element.pre_conditions().0 {
			for i in 0..PRE_CONDITIONS.len() {
				if let Some(cond) = &conditions[i] {
					writer.attr(PRE_CONDITIONS[i], cond)?;
				}
			}
		}

		// Post-conditions
		if let Some(conditions) = &element.post_conditions().0 {
			for i in 0..POST_CONDITIONS.len() {
				if let Some(cond) = &conditions[i] {
					writer.attr(POST_CONDITIONS[i], cond)?;
				}
			}
		}

		if !is_subtree {
			// recursive dive into children, ignoring subtrees
			for element in element.children().iter() {
				Self::write_subtree(element, writer, metadata)?;
			}
		}

		writer.end_elem()?;

		Ok(())
	}

	/// Create XML from tree including `TreeNodesModel`.
	/// # Errors
	/// # Panics
	pub fn groot_write_tree(tree: &BehaviorTree) -> Result<bytes::Bytes, woxml::Error> {
		// storage for (non groot2 builtin) behaviors to mention in TreeNodesModel
		let mut behaviors: BTreeMap<ConstString, BehaviorDescription> = BTreeMap::new();
		let mut subtrees: BTreeMap<ConstString, &BehaviorTreeElement> = BTreeMap::new();

		let mut writer = XmlWriter::compact_mode(bytes::BytesMut::new());
		{
			writer.begin_elem("root")?;
			writer.attr("BTCPP_format", "4")?;

			// scan the tree
			for item in tree.iter() {
				#[allow(clippy::match_same_arms)]
				match item.kind() {
					TreeElementKind::Leaf => {
						let desc = item.data().description();
						behaviors.insert(desc.name().clone(), desc.clone());
					}
					TreeElementKind::Node => {
						let desc = item.data().description();
						if !desc.groot2() {
							behaviors.insert(desc.name().clone(), desc.clone());
						}
					}
					TreeElementKind::SubTree => {
						subtrees.insert(item.data().description().path().clone(), item);
					}
				}
			}

			// create the BehaviorTree's
			for (_path, subtree) in subtrees {
				writer.begin_elem("BehaviorTree")?;
				writer.attr("ID", subtree.data().description().name())?;
				writer.attr("_fullpath", subtree.data().description().groot2_path())?;

				// recursive dive into children
				for element in subtree.children().iter() {
					Self::groot_write_subtree(element, &mut writer)?;
				}
				writer.end_elem()?; // BehaviorTree
			}

			// create the TreeNodesModel
			writer.begin_elem("TreeNodesModel")?;
			// loop over collected behavior entries
			for (name, item) in &behaviors {
				writer.begin_elem(item.kind_str())?;
				writer.attr("ID", name)?;
				// look for a PortsList
				for port in &item.ports().0 {
					writer.begin_elem(port.direction().type_str())?;
					writer.attr("name", port.name())?;
					writer.attr("type", Self::groot_map_types(port.type_name()))?;
					if !port.description().is_empty() {
						writer.text(port.description())?;
					}
					writer.end_elem()?;
				}
				writer.end_elem()?;
			}

			writer.end_elem()?; // TreeNodesModel
			writer.end_elem()?; // root
			writer.flush()?;
		}

		let inner = writer.into_inner();
		Ok(inner.into())
	}

	fn groot_write_subtree<'a>(
		element: &'a BehaviorTreeElement,
		writer: &mut XmlWriter<'a, bytes::BytesMut>,
	) -> Result<(), woxml::Error> {
		let is_subtree = match element.kind() {
			TreeElementKind::Leaf | TreeElementKind::Node => {
				writer.begin_elem(element.data().description().id())?;
				writer.attr("name", element.data().description().name())?;
				false
			}
			TreeElementKind::SubTree => {
				writer.begin_elem("SubTree")?;
				writer.attr("ID", element.data().description().name())?;
				writer.attr("_fullpath", element.data().description().groot2_path())?;
				true
			}
		};
		writer.attr("_uid", &element.data().uid().to_string())?;

		if is_subtree {
			// subtree port mappings/values are in blackboard
			if let Some(remappings) = element.data().blackboard().remappings() {
				for remapping in remappings.iter() {
					writer.attr(&remapping.0, &remapping.1)?;
				}
			}
		} else {
			// behavior port mappings/values
			for remapping in element.data().remappings().iter() {
				writer.attr(&remapping.0, &remapping.1)?;
			}
		}

		// Pre-conditions
		if let Some(conditions) = &element.pre_conditions().0 {
			for i in 0..PRE_CONDITIONS.len() {
				if let Some(cond) = &conditions[i] {
					writer.attr(PRE_CONDITIONS[i], cond)?;
				}
			}
		}

		// Post-conditions
		if let Some(conditions) = &element.post_conditions().0 {
			for i in 0..POST_CONDITIONS.len() {
				if let Some(cond) = &conditions[i] {
					writer.attr(POST_CONDITIONS[i], cond)?;
				}
			}
		}

		if !is_subtree {
			// recursive dive into children, ignoring subtrees
			for element in element.children().iter() {
				Self::groot_write_subtree(element, writer)?;
			}
		}

		writer.end_elem()?;

		Ok(())
	}

	// @TODO: things like: SharedQueue<T: FromStr + ToString>(pub Arc<Mutex<VecDeque<T>>>);
	fn groot_map_types(input: &str) -> &str {
		match input {
			"char" => input,
			"i16" => "short",
			"u16" => "unsigned short",
			"i32" => "int",
			"u32" => "unsigned int",
			"i64" => "long",
			"u64" => "unsigned long",
			"f32" => "float",
			"f64" => "double",
			"String" => "std::string",
			"BehaviorState" => "BT::NodeStatus",
			_ => "BT::Any",
		}
	}
}
// endregion:   --- XmlWriter
