// Copyright © 2025 Stephan Kunz

//! XML parser for the [`BehaviorTreeFactory`]

#[doc(hidden)]
#[cfg(feature = "std")]
extern crate std;

// region:      --- modules
use alloc::{
	boxed::Box,
	string::{String, ToString},
};
use roxmltree::{Document, Node, NodeType};
#[cfg(feature = "std")]
use std::path::PathBuf;
use tracing::{Level, event, instrument};

use crate::{
	ConstString, EMPTY_STR, ID, NAME, SUBTREE,
	behavior::{
		BehaviorData, BehaviorDescription, BehaviorExecution, BehaviorKind, BehaviorPtr,
		pre_post_conditions::{Conditions, PostConditions, PreConditions},
	},
	blackboard::SharedBlackboard,
	factory::registry::BehaviorRegistry,
	port::{PortRemappings, is_allowed_port_name, strip_bb_pointer},
	tree::{tree_element::BehaviorTreeElement, tree_element_list::BehaviorTreeElementList},
	xml::error::Error,
};
// endregion:   --- modules

// region:		--- helper
fn handle_attributes(
	name: &str,
	is_subtree: bool,
	bhvr: &BehaviorPtr,
	node: &Node,
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
						match remappings.add(port_definition.name(), default_value.clone()) {
							Ok(()) => {}
							Err(err) => return Err(Error::Remapping(err)),
						}
					} else {
						return Err(Error::NameNotAllowed(port_definition.name().to_string().into()));
					}
				}
				// No bb pointer
				None => match remappings.add(port_definition.name(), default_value.clone()) {
					Ok(()) => {}
					Err(err) => return Err(Error::Remapping(err)),
				},
			}
		}
	}
	// handle attributes
	for attribute in node.attributes() {
		let key = attribute.name();
		let value = attribute.value();
		if key == NAME {
			// port "name" is always available
		} else if key == ID {
			// ignore as it is not a Port
		} else if key.starts_with('_') {
			// these are special attributes
			match key {
				crate::AUTOREMAP => {
					autoremap = match attribute.value().parse::<bool>() {
						Ok(val) => val,
						Err(_) => return Err(Error::WrongAutoremap)?,
					};
				}
				// preconditions
				crate::FAILURE_IF | crate::SKIP_IF | crate::SUCCESS_IF | crate::WHILE => {
					match preconditions.set(key, value) {
						Ok(()) => {}
						Err(err) => return Err(Error::Precondition(key.into(), err)),
					}
				}
				// postconditions
				crate::ON_FAILURE | crate::ON_HALTED | crate::ON_SUCCESS | crate::POST => {
					match postconditions.set(key, value) {
						Ok(()) => {}
						Err(err) => return Err(Error::Postcondition(key.into(), err)),
					}
				}
				_ => return Err(Error::UnknownSpecialAttribute(key.into()))?,
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
						return Err(Error::NameNotAllowed(stripped.into()));
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
									return Err(Error::NameNotAllowed(stripped));
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
						return Err(Error::PortInvalid(key.into(), name.into(), port_list.entries()));
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
// endregion:	--- DataItem

// region:		--- DataItem
/// This is used to minimize the stack consumption during recursion of tree creation.
struct DataItem {
	bhvr_desc: BehaviorDescription,
	bhvr: Box<dyn BehaviorExecution>,
	remappings: PortRemappings,
	conditions: Conditions,
	autoremap: bool,
}

impl DataItem {
	fn create(bhvr_name: &str, is_subtree: bool, registry: &BehaviorRegistry, node: &Node) -> Result<Box<Self>, Error> {
		// look for the behavior in the `BehaviorRegistry`
		let res = if is_subtree {
			registry.fetch(SUBTREE)
		} else {
			registry.fetch(bhvr_name)
		};
		let Ok((bhvr_desc, bhvr_creation_fn)) = res else {
			return Err(Error::BehaviorNotRegistered(bhvr_name.into()));
		};
		let bhvr = bhvr_creation_fn();
		let (autoremap, remappings, conditions) = handle_attributes(bhvr_name, is_subtree, &bhvr, node)?;
		Ok(Box::new(Self {
			bhvr_desc,
			bhvr,
			remappings,
			conditions,
			autoremap,
		}))
	}
}
// endregion:	--- DataItem

// region:      --- XmlParser
#[derive(Default)]
pub struct XmlParser {
	uid: u16,
}

impl XmlParser {
	/// Get the next uid for a [`BehaviorTreeElement`].
	/// # Panics
	/// if more than 65536 [`BehaviorTreeElement`]s are required for a [`BehaviorTree`](crate::tree::tree::BehaviorTree)
	const fn next_uid(&mut self) -> u16 {
		let next = self.uid;
		self.uid += 1;
		next
	}

	#[instrument(level = Level::DEBUG, skip_all)]
	pub(crate) fn register_document(registry: &mut BehaviorRegistry, xml: &ConstString) -> Result<(), Error> {
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

		Self::register_document_root(registry, root, xml)?;
		Ok(())
	}

	#[instrument(level = Level::DEBUG, skip_all)]
	fn register_document_root(registry: &mut BehaviorRegistry, element: Node, source: &ConstString) -> Result<(), Error> {
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
							if let Some(id) = element.attribute(ID) {
								// if no explicit main tree id is given, the first found id will be used for main tree
								if registry.main_tree_id().is_none() {
									registry.set_main_tree_id(id);
								}
								// let source: ConstString = element.document().input_text()[element.range()].into();
								match registry.add_tree_defintion(id, source.clone(), element.range()) {
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
							let xml = std::fs::read_to_string(file_path)?.into();
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
			|(definition, range)| {
				// @TODO embedded: use same mechanism for both -> manual conversion of error!!
				#[cfg(feature = "std")]
				let doc = Document::parse(&definition[range])?;
				#[cfg(not(feature = "std"))]
				let doc = match Document::parse(&definition[range]) {
					Ok(doc) => doc,
					Err(_err) => return Err(Error::XmlParser).into(),
				};
				let node = doc.root_element();
				let uid = self.next_uid();
				let mut data = DataItem::create(SUBTREE, true, registry, &node)?;
				let blackboard = if let Some(external_bb) = external_blackboard {
					// in this case, the remappings are against parent BlackBoard
					let mut remappings = PortRemappings::default();
					core::mem::swap(&mut remappings, &mut data.remappings);
					SharedBlackboard::with_parent(name, external_bb, remappings, data.autoremap)
				} else {
					SharedBlackboard::new(name)
				};
				// for tree root "path" is empty
				let children = self.build_children(EMPTY_STR, node, registry, &blackboard)?;
				if children.len() > 1 {
					return Err(Error::SubtreeOneChild(node.tag_name().name().into()));
				}
				let bhvr_data = BehaviorData::new(uid, name, EMPTY_STR, data.remappings, blackboard, data.bhvr_desc);
				let behaviortree =
					BehaviorTreeElement::create_subtree(bhvr_data, children.into(), data.bhvr, data.conditions);
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
		let is_subtree = tag_name == SUBTREE;

		// if node is denoted with type of behavior, use attribute "ID" as name
		if tag_name == crate::ACTION
			|| tag_name == crate::CONDITION
			|| tag_name == crate::CONTROL
			|| tag_name == crate::DECORATOR
			|| tag_name == crate::SUBTREE
		{
			if let Some(id) = node.attribute(ID) {
				tag_name = id;
			} else {
				return Err(Error::MissingId(node.tag_name().name().into()))?;
			}
		}

		// if node has no assigned name, use tag name
		let node_name = node
			.attribute(NAME)
			.map_or_else(|| String::from(tag_name), ToString::to_string);
		let mut path = String::from(path) + "/" + &node_name;
		// in case no explicit name was given, we extend the node_name with the uid
		if node.attribute(NAME).is_none() {
			path.push_str("::");
			path.push_str(&uid.to_string());
		}

		let data = DataItem::create(tag_name, is_subtree, registry, &node)?;
		// ariel_os::debug::log::info!("build child: {}", node_name.as_str());
		let tree_node = match data.bhvr_desc.kind() {
			BehaviorKind::Action | BehaviorKind::Condition => {
				// A leaf uses a cloned Blackboard
				if node.has_children() {
					return Err(Error::ChildrenNotAllowed(node_name.into()))?;
				}
				let bhvr_data = BehaviorData::new(uid, &node_name, &path, data.remappings, blackboard, data.bhvr_desc);
				BehaviorTreeElement::create_leaf(bhvr_data, data.bhvr, data.conditions)
			}
			BehaviorKind::Control | BehaviorKind::Decorator => {
				// A node uses a cloned Blackboard
				let children = self.build_children(&path, node, registry, &blackboard)?;

				if data.bhvr_desc.kind() == BehaviorKind::Decorator && children.len() != 1 {
					return Err(Error::DecoratorOneChild(node.tag_name().name().into()))?;
				}
				let bhvr_data = BehaviorData::new(uid, &node_name, &path, data.remappings, blackboard, data.bhvr_desc);
				BehaviorTreeElement::create_node(bhvr_data, children.into(), data.bhvr, data.conditions)
			}
			BehaviorKind::SubTree => {
				if let Some(id) = node.attribute(ID) {
					match registry.find_tree_definition(id) {
						Some((definition, range)) => {
							// @TODO embedded: use same mechanism for both -> manual conversion of error!!
							#[cfg(feature = "std")]
							let doc = Document::parse(&definition[range])?;
							#[cfg(not(feature = "std"))]
							let doc = match Document::parse(&definition[range]) {
								Ok(doc) => doc,
								Err(_err) => return Err(Error::XmlParser),
							};
							let node = doc.root_element();
							// A SubTree gets a new Blackboard with parent and remappings.
							let blackboard1 =
								SharedBlackboard::with_parent(&node_name, blackboard, data.remappings, data.autoremap);
							let children = self.build_children(&path, node, registry, &blackboard1)?;
							if children.len() > 1 {
								return Err(Error::SubtreeOneChild(node.tag_name().name().into()))?;
							}
							// the PortRemappings have been used against parent BlackBoard
							let bhvr_data = BehaviorData::new(
								uid,
								&node_name,
								&path,
								PortRemappings::default(),
								blackboard1,
								data.bhvr_desc,
							);
							BehaviorTreeElement::create_subtree(bhvr_data, children.into(), data.bhvr, data.conditions)
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
