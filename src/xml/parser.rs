// Copyright © 2025 Stephan Kunz

//! XML parser for the [`BehaviorTreeFactory`]

#[doc(hidden)]
#[cfg(feature = "std")]
extern crate std;

// region:      --- modules
use crate::{ConstString, EMPTY_STR, port::strip_bb_pointer};
use alloc::{
	collections::btree_map::BTreeMap,
	string::{String, ToString},
};
use roxmltree::{Attributes, Document, Node, NodeType};
#[cfg(feature = "std")]
use std::path::PathBuf;
use tracing::{Level, event, instrument};

use crate::{
	behavior::{
		BehaviorData, BehaviorKind, BehaviorPtr,
		pre_post_conditions::{Conditions, PostConditions, PreConditions},
	},
	blackboard::SharedBlackboard,
	factory::registry::BehaviorRegistry,
	port::{PortRemappings, is_allowed_port_name},
	tree::{tree_element::BehaviorTreeElement, tree_element_list::BehaviorTreeElementList},
	xml::error::Error,
};
// endregion:   --- modules

// region:		--- helper
fn attrs_to_map(attrs: Attributes) -> BTreeMap<ConstString, ConstString> {
	let mut map = BTreeMap::default();
	for attr in attrs {
		let name = attr.name().into();
		let value = attr.value().into();
		map.insert(name, value);
	}
	map
}
// endregion:	--- helper

// region:      --- XmlParser
#[derive(Default)]
pub struct XmlParser {
	uid: u16,
}

impl XmlParser {
	/// Get the next uid for a [`BehaviorTreeElement`].
	/// # Panics
	/// if more than 65536 [`BehaviorTreeElement`]s are required for a [`BehaviorTree`](crate::tree::BehaviorTree)
	const fn next_uid(&mut self) -> u16 {
		let next = self.uid;
		self.uid += 1;
		next
	}

	#[instrument(level = Level::DEBUG, skip_all)]
	pub(crate) fn register_document(registry: &mut BehaviorRegistry, xml: &str) -> Result<(), Error> {
		// general checks
		// @TODO embedded: use same mechanism for both -> manual conversion of error!!
		#[cfg(feature = "std")]
		let doc = Document::parse(xml)?;
		#[cfg(not(feature = "std"))]
		let doc = match Document::parse(xml) {
			Ok(doc) => doc,
			Err(_err) => return Err(Error::XmlParser),
		};
		let root = doc.root_element();
		if root.tag_name().name() != "root" {
			return Err(Error::WrongRootName)?;
		}
		if let Some(format) = root.attribute("BTCPP_format") {
			if format != "4" {
				return Err(Error::BtCppFormat)?;
			}
		}

		// handle the attribute 'main_tree_to_execute`
		if let Some(name) = root.attribute("main_tree_to_execute") {
			registry.set_main_tree_id(name);
		}

		Self::register_document_root(registry, root)?;
		Ok(())
	}

	#[instrument(level = Level::DEBUG, skip_all)]
	fn register_document_root(registry: &mut BehaviorRegistry, element: Node) -> Result<(), Error> {
		event!(Level::TRACE, "register_document_root");
		for element in element.children() {
			match element.node_type() {
				NodeType::Comment | NodeType::Text => {} // ignore
				NodeType::Root => {
					// this should not happen
					return Err(Error::Unexpected("root element".into(), file!().into(), line!()))?;
				}
				NodeType::Element => {
					// only 'BehaviorTree' or 'TreeNodesModel' are valid
					let name = element.tag_name().name();
					match name {
						"TreeNodesModel" => {} // ignore
						"BehaviorTree" => {
							// check for tree ID
							if let Some(id) = element.attribute("ID") {
								// if no explicit main tree id is given, the first found id will be used for main tree
								if registry.main_tree_id().is_none() {
									registry.set_main_tree_id(id);
								}
								let source: ConstString = element.document().input_text()[element.range()].into();
								match registry.add_tree_defintion(id, source) {
									Ok(()) => {}
									Err(err) => return Err(Error::Registration(id.into(), err)),
								}
							} else {
								return Err(Error::MissingId(element.tag_name().name().into()))?;
							}
						}
						#[cfg(feature = "std")]
						"include" => {
							let mut file_path: PathBuf;
							if let Some(path) = element.attribute("path") {
								file_path = PathBuf::from(path);
								if file_path.is_relative() {
									// get the "current" directory
									file_path = std::env::current_dir()?;
									file_path.push(path);
								}
							} else {
								return Err(Error::MissingPath(element.tag_name().name().into()))?;
							}
							let xml = std::fs::read_to_string(file_path)?;
							Self::register_document(registry, &xml)?;
						}
						_ => {
							return Err(Error::ElementNotSupported(element.tag_name().name().into()))?;
						}
					}
				}
				NodeType::PI => {
					return Err(Error::UnsupportedProcessingInstruction(element.tag_name().name().into()))?;
				}
			}
		}
		Ok(())
	}

	fn handle_attributes(
		name: &str,
		is_subtree: bool,
		bhvr: &BehaviorPtr,
		attrs: &BTreeMap<ConstString, ConstString>,
	) -> Result<
		(
			/*autoremap:*/ bool,
			/*remappings:*/ PortRemappings,
			/*pre&post conditions:*/ Conditions,
		),
		Error,
	> {
		let mut autoremap = false;
		let mut remappings = PortRemappings::default();
		let mut preconditions = PreConditions::default();
		let mut postconditions = PostConditions::default();

		// port list is needed twice:
		// - for checking port names in given attributes
		// - to add default values
		let port_list = bhvr.static_provided_ports();
		// first check for default values given in port definition.
		// this value can later be overwritten by default values given by xml attribute
		for port_definition in port_list.iter() {
			if let Some(default_value) = port_definition.default_value() {
				// check if it is a BB pointer
				match strip_bb_pointer(default_value) {
					// Bb pointer
					Some(stripped) => {
						if stripped.as_ref() == "=" {
							// remapping to itself not necessary
						} else if is_allowed_port_name(&stripped) {
							match remappings.add(port_definition.name(), default_value) {
								Ok(()) => {}
								Err(err) => return Err(Error::Remapping(err)),
							}
						} else {
							return Err(Error::NameNotAllowed(port_definition.name().to_string().into()));
						}
					}
					// No bb pointer
					None => match remappings.add(port_definition.name(), default_value) {
						Ok(()) => {}
						Err(err) => return Err(Error::Remapping(err)),
					},
				}
			}
		}
		// handle attributes
		for (key, value) in attrs {
			if key.as_ref() == "name" {
				// port "name" is always available
			} else if key.as_ref() == "ID" {
				// ignore as it is not a Port
			} else if key.starts_with('_') {
				// these are special attributes
				match key.as_ref() {
					"_autoremap" => {
						autoremap = match value.parse::<bool>() {
							Ok(val) => val,
							Err(_) => return Err(Error::WrongAutoremap)?,
						};
					}
					// preconditions
					"_skipif" | "_failureif" | "_successif" | "_while" => match preconditions.set(key, value) {
						Ok(()) => {}
						Err(err) => return Err(Error::Precondition(key.clone(), err)),
					},
					// postconditions
					"_onSuccess" | "_onFailure" | "_post" | "_onHalted" => match postconditions.set(key, value) {
						Ok(()) => {}
						Err(err) => return Err(Error::Postcondition(key.clone(), err)),
					},
					_ => return Err(Error::UnknownSpecialAttribute(key.clone()))?,
				}
			} else {
				// for a subtree we cannot check the ports
				if is_subtree {
					// check if it is a BB pointer
					if value.starts_with('{') && value.ends_with('}') {
						let stripped = value
							.strip_prefix('{')
							.unwrap_or_else(|| todo!())
							.strip_suffix('}')
							.unwrap_or_else(|| todo!());

						// check value for allowed names
						if is_allowed_port_name(stripped) {
							remappings.overwrite(key, value);
						} else {
							return Err(Error::NameNotAllowed(stripped.into()))?;
						}
					} else {
						// this is a normal string, representing a port value
						remappings.overwrite(key, value);
					}
				} else {
					// check found port name against list of provided ports
					match port_list.find(key) {
						Some(_) => {
							// check if it is a BB pointer
							match strip_bb_pointer(value) {
								// Bb pointer
								Some(stripped) => {
									// check stripped value for allowed names
									if is_allowed_port_name(&stripped) {
										remappings.overwrite(key, value);
									} else {
										return Err(Error::NameNotAllowed(stripped))?;
									}
								}
								// No bb pointer
								None => {
									// this is a normal string, representing a port value
									remappings.overwrite(key, value);
								}
							}
						}
						None => {
							return Err(Error::PortInvalid(key.clone(), name.into(), port_list.entries()))?;
						}
					}
				}
			}
		}
		remappings.shrink();
		let conditions = Conditions {
			pre: preconditions,
			post: postconditions,
		};
		Ok((autoremap, remappings, conditions))
	}

	#[allow(clippy::option_if_let_else)]
	#[instrument(level = Level::DEBUG, skip_all)]
	pub(crate) fn create_tree_from_definition(
		&mut self,
		name: &str,
		registry: &mut BehaviorRegistry,
		external_blackboard: Option<SharedBlackboard>,
	) -> Result<BehaviorTreeElement, Error> {
		event!(Level::TRACE, "create_tree_from_definition");
		registry.find_tree_definition(name).map_or_else(
			|| Err(Error::SubtreeNotFound(name.into())),
			|definition| {
				// @TODO embedded: use same mechanism for both -> manual conversion of error!!
				#[cfg(feature = "std")]
				let doc = Document::parse(&definition)?;
				#[cfg(not(feature = "std"))]
				let doc = match Document::parse(&definition) {
					Ok(doc) => doc,
					Err(_err) => return Err(Error::XmlParser).into(),
				};
				let node = doc.root_element();
				// look for the "SubTree" behavior in the `BehaviorRegistry` and create it.
				let Ok((bhvr_desc, bhvr_creation_fn)) = registry.fetch("SubTree") else {
					return Err(Error::BehaviorNotRegistered("SubTree".into()));
				};
				let bhvr = bhvr_creation_fn();
				let uid = self.next_uid();
				// handle the nodes attributes
				let attrs = attrs_to_map(node.attributes());
				let (autoremap, mut remappings, conditions) = Self::handle_attributes(name, true, &bhvr, &attrs)?;
				let blackboard = if let Some(external_bb) = external_blackboard {
					// in this case, the remappings are against parent BlackBoard
					let bb = SharedBlackboard::with_parent(name, external_bb, remappings, autoremap);
					remappings = PortRemappings::default();
					bb
				} else {
					SharedBlackboard::new(name)
				};
				// for tree root "path" is empty
				let children = self.build_children("", node, registry, &blackboard)?;
				if children.len() > 1 {
					return Err(Error::SubtreeOneChild(node.tag_name().name().into()));
				}
				let bhvr_data = BehaviorData::new(uid, name, EMPTY_STR, remappings, blackboard, bhvr_desc);
				let behaviortree = BehaviorTreeElement::create_subtree(bhvr_data, children.into(), bhvr, conditions);
				Ok(behaviortree)
			},
		)
	}

	#[instrument(level = Level::DEBUG, skip_all)]
	fn build_children(
		&mut self,
		path: &str,
		node: Node,
		registry: &mut BehaviorRegistry,
		blackboard: &SharedBlackboard,
	) -> Result<BehaviorTreeElementList, Error> {
		event!(Level::TRACE, "build_children");
		let mut children = BehaviorTreeElementList::default();
		for child in node.children() {
			match child.node_type() {
				NodeType::Comment | NodeType::Text => {} // ignore
				NodeType::Root => {
					// this should not happen
					return Err(Error::Unexpected("root element".into(), file!().into(), line!()))?;
				}
				NodeType::Element => {
					let element = self.build_child(path, child, registry, blackboard.clone())?;
					children.push(element);
				}
				NodeType::PI => {
					return Err(Error::UnsupportedProcessingInstruction(node.tag_name().name().into()))?;
				}
			}
		}

		children.shrink_to_fit();
		Ok(children)
	}

	#[instrument(level = Level::DEBUG, skip_all)]
	fn build_child(
		&mut self,
		path: &str,
		node: Node,
		registry: &mut BehaviorRegistry,
		blackboard: SharedBlackboard,
	) -> Result<BehaviorTreeElement, Error> {
		event!(Level::TRACE, "build_child");
		let uid = self.next_uid();
		let mut tag_name = node.tag_name().name();
		let is_subtree = tag_name == "SubTree";
		// handle the nodes attributes
		let attrs = attrs_to_map(node.attributes());

		// if node is denoted with type of behavior, use ID attribute as name
		if tag_name == "Action"
			|| tag_name == "Condition"
			|| tag_name == "Control"
			|| tag_name == "Decorator"
			|| tag_name == "SubTree"
		{
			if let Some(id) = attrs.get("ID") {
				tag_name = id;
			} else {
				return Err(Error::MissingId(node.tag_name().name().into()))?;
			}
		}

		// if node has no assigned name, use tag name
		let node_name = attrs
			.get("name")
			.map_or_else(|| String::from(tag_name), ToString::to_string);

		let mut path = String::from(path) + "/" + &node_name;
		// in case no explicit name was given, we extend the node_name with the uid
		if !attrs.contains_key("name") {
			path.push_str("::");
			path.push_str(&uid.to_string());
		}

		// look for the behavior in the `BehaviorRegistry`
		let res = if is_subtree {
			registry.fetch("SubTree")
		} else {
			registry.fetch(tag_name)
		};
		let Ok((bhvr_desc, bhvr_creation_fn)) = res else {
			return Err(Error::BehaviorNotRegistered(tag_name.into()));
		};
		let bhvr = bhvr_creation_fn();
		let (autoremap, remappings, conditions) = Self::handle_attributes(&node_name, is_subtree, &bhvr, &attrs)?;
		let kind = bhvr_desc.kind();
		let tree_node = match kind {
			BehaviorKind::Action | BehaviorKind::Condition => {
				if node.has_children() {
					return Err(Error::ChildrenNotAllowed(node_name.into()))?;
				}
				// A leaf uses a cloned Blackboard
				let bhvr_data = BehaviorData::new(uid, &node_name, &path, remappings, blackboard, bhvr_desc);
				BehaviorTreeElement::create_leaf(bhvr_data, bhvr, conditions)
			}
			BehaviorKind::Control | BehaviorKind::Decorator => {
				// A node uses a cloned Blackboard
				let children = self.build_children(&path, node, registry, &blackboard)?;

				if kind == BehaviorKind::Decorator && children.len() != 1 {
					return Err(Error::DecoratorOneChild(node.tag_name().name().into()))?;
				}
				let bhvr_data = BehaviorData::new(uid, &node_name, &path, remappings, blackboard, bhvr_desc);
				BehaviorTreeElement::create_node(bhvr_data, children.into(), bhvr, conditions)
			}
			BehaviorKind::SubTree => {
				if let Some(id) = attrs.get("ID") {
					let definition = registry.find_tree_definition(id);
					match definition {
						Some(definition) => {
							// @TODO embedded: use same mechanism for both -> manual conversion of error!!
							#[cfg(feature = "std")]
							let doc = Document::parse(&definition)?;
							#[cfg(not(feature = "std"))]
							let doc = match Document::parse(&definition) {
								Ok(doc) => doc,
								Err(_err) => return Err(Error::XmlParser),
							};
							let node = doc.root_element();
							// A SubTree gets a new Blackboard with parent and remappings.
							let blackboard1 = SharedBlackboard::with_parent(&node_name, blackboard, remappings, autoremap);
							let children = self.build_children(&path, node, registry, &blackboard1)?;
							if children.len() > 1 {
								return Err(Error::SubtreeOneChild(node.tag_name().name().into()))?;
							}
							// the PortRemappings have been used against parent BlackBoard
							let bhvr_data =
								BehaviorData::new(uid, &node_name, &path, PortRemappings::default(), blackboard1, bhvr_desc);
							BehaviorTreeElement::create_subtree(bhvr_data, children.into(), bhvr, conditions)
						}
						None => {
							return Err(Error::SubtreeNotFound(node_name.into()))?;
						}
					}
				} else {
					return Err(Error::MissingId(node.tag_name().name().into()))?;
				}
			}
		};
		Ok(tree_node)
	}
}
// endregion:   --- XmlParser
