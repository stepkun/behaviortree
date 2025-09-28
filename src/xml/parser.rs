// Copyright © 2025 Stephan Kunz

//! XML parser for the [`BehaviorTreeFactory`]

#[doc(hidden)]
#[cfg(feature = "std")]
extern crate std;

use alloc::{
	boxed::Box,
	string::{String, ToString},
};
// region:      --- modules
use crate::{
	ACTION, BEHAVIORTREE, CONDITION, CONTROL, ConstString, DECORATOR, DEFAULT, EMPTY_STR, ID, NAME, SUBTREE, TREENODESMODEL,
	behavior::{BehaviorDataCollection, BehaviorKind, BehaviorPtr, pre_post_conditions::Conditions},
	factory::registry::{BehaviorRegistry, TreeNodesModelEntry},
	port::{PortDirection, is_allowed_port_name},
	tree::{BehaviorTreeElement, BehaviorTreeElementList},
	xml::error::Error,
};
use databoard::{Databoard, Remappings, strip_board_pointer};
use roxmltree::{Document, Node, NodeType};
#[cfg(feature = "std")]
use std::path::PathBuf;
use tracing::{Level, event, instrument};
// endregion:   --- modules

// region:		--- helper
fn create_data_collection_from_xml(
	registry: &BehaviorRegistry,
	path: &str,
	element: &Node,
	uid: u16,
	blackboard: Option<&Databoard>,
	is_root: bool,
) -> Result<Box<BehaviorDataCollection>, Error> {
	let (behavior_id, behavior_kind) = {
		let tag_name = element.tag_name().name();
		match tag_name {
			BEHAVIORTREE => {
				if let Some(id) = element.attribute(ID) {
					(id, SUBTREE)
				} else {
					return Err(Error::MissingId { tag: tag_name.into() });
				}
			}
			ACTION | CONDITION | CONTROL | DECORATOR | SUBTREE => {
				if let Some(id) = element.attribute(ID) {
					(id, tag_name)
				} else {
					return Err(Error::MissingId { tag: tag_name.into() });
				}
			}
			_ => (tag_name, EMPTY_STR),
		}
	};
	let is_subtree = behavior_kind == SUBTREE;

	// if behavior has no assigned name, use beavior id
	let behavior_name = element
		.attribute(NAME)
		.map_or_else(|| behavior_id.to_string(), ToString::to_string);
	let mut path = String::from(path) + "/" + &behavior_name;
	// in case no explicit name was given, we extend the node_name with the uid
	if element.attribute(NAME).is_none() {
		path.push_str("::");
		path.push_str(&uid.to_string());
	}

	// look for the behavior in the `BehaviorRegistry`
	let res = if is_subtree {
		registry.fetch(SUBTREE)
	} else {
		registry.fetch(behavior_id)
	};
	let Ok((mut bhvr_desc, bhvr_creation_fn)) = res else {
		return Err(Error::NotRegistered {
			behavior: behavior_id.into(),
		});
	};
	bhvr_desc.set_name(&behavior_name);
	bhvr_desc.set_path(&path);

	let bhvr = bhvr_creation_fn();
	let (autoremap, mut remappings, conditions) = handle_attributes(registry, behavior_id, behavior_kind, &bhvr, element)?;

	let blackboard = blackboard.map_or_else(Databoard::new, |blackboard| {
		if is_subtree && !is_root {
			// A SubTree gets a new Blackboard with parent and remappings.
			let mut new_remappings = Remappings::default();
			core::mem::swap(&mut new_remappings, &mut remappings);
			Databoard::with(Some(blackboard.clone()), Some(new_remappings), autoremap)
		} else {
			blackboard.clone()
		}
	});

	Ok(Box::new(BehaviorDataCollection {
		behavior_name,
		path,
		bhvr_desc,
		blackboard,
		bhvr,
		remappings,
		conditions,
		uid,
	}))
}

#[allow(clippy::too_many_lines)]
fn handle_attributes(
	registry: &BehaviorRegistry,
	behavior_id: &str,
	behavior_kind: &str,
	bhvr: &BehaviorPtr,
	node: &Node,
) -> Result<
	(
		/*autoremap:*/ bool,
		/*remappings:*/ Remappings,
		/*pre&post conditions:*/ Conditions,
	),
	Error,
> {
	let mut autoremap = false;
	let mut remappings = Remappings::default();
	let mut conditions = Conditions::default();
	// let mut preconditions = PreConditions::default();
	// let mut postconditions = PostConditions::default();

	// port list is needed twice:
	// - for checking port names in given attributes
	// - to add default values
	let port_list = bhvr.static_provided_ports();

	// first check for default values given in port definition.
	// this value can later be overwritten by default values given by xml attribute
	for port_definition in port_list.iter() {
		if let Some(default_value) = port_definition.default_value() {
			match remappings.add(port_definition.name(), default_value.clone()) {
				Ok(()) => {}
				Err(err) => {
					return Err(Error::Databoard {
						key: port_definition.name().into(),
						source: err,
					});
				}
			}
		}
	}

	// second fill in remappings from available TreeNodesModel's
	for entry in registry.tree_nodes_models() {
		if entry.0.contains(behavior_id) {
			match remappings.add(entry.1.key.clone(), entry.1.remapping.clone()) {
				Ok(()) => {}
				Err(err) => {
					return Err(Error::Databoard {
						key: entry.1.key.clone(),
						source: err,
					});
				}
			}
		}
	}

	// third handle attributes
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
						Err(_) => return Err(Error::WrongAutoremap),
					};
				}
				// preconditions
				crate::FAILURE_IF | crate::SKIP_IF | crate::SUCCESS_IF | crate::WHILE => {
					match conditions.pre.set(key, value) {
						Ok(()) => {}
						Err(err) => {
							return Err(Error::Condition {
								key: key.into(),
								source: err,
							});
						}
					}
				}
				// postconditions
				crate::ON_FAILURE | crate::ON_HALTED | crate::ON_SUCCESS | crate::POST => {
					match conditions.post.set(key, value) {
						Ok(()) => {}
						Err(err) => {
							return Err(Error::Condition {
								key: key.into(),
								source: err,
							});
						}
					}
				}
				_ => return Err(Error::UnknownAttribute { key: key.into() }),
			}
		} else {
			// for a subtree we cannot check against a port list
			if behavior_kind == SUBTREE {
				remappings.overwrite(key, value);
			} else {
				// check key against list of provided ports
				match port_list.find(key) {
					Some(_port) => {
						match strip_board_pointer(value) {
							Some(stripped) => {
								if stripped == "=" {
									if is_allowed_port_name(key) {
										let bb_pointer = String::from("{") + key + "}";
										remappings.overwrite(key, bb_pointer);
									} else {
										return Err(Error::NameNotAllowed { key: key.into() });
									}
								} else {
									// check if 'value' contains a valid BB pointer
									if is_allowed_port_name(stripped) {
										remappings.overwrite(key, value);
									} else {
										return Err(Error::NameNotAllowed { key: key.into() });
									}
								}
							}
							// Normal string, representing a const assignment
							None => {
								remappings.overwrite(key, value);
							}
						}
					}
					None => {
						return Err(Error::PortInvalid {
							port: key.into(),
							behavior: behavior_id.into(),
						});
					}
				}
			}
		}
	}
	remappings.shrink();
	Ok((autoremap, remappings, conditions))
}
// endregion:	--- helper

// region:      --- XmlParser
#[derive(Default)]
pub struct XmlParser {
	uid: u16,
}

impl XmlParser {
	/// Get the next uid for a [`BehaviorTreeElement`].
	/// The maximum allowed number of behaviors in a tree is 65535!
	/// # Panics
	/// - if more than 65535 [`BehaviorTreeElement`]s are created for a [`BehaviorTree`](crate::tree::tree::BehaviorTree)
	const fn next_uid(&mut self) -> u16 {
		let next = self.uid;
		self.uid += 1;
		next
	}

	#[instrument(level = Level::DEBUG, skip_all)]
	pub(crate) fn register_document(
		registry: &mut BehaviorRegistry,
		xml: &str,
		#[cfg(feature = "std")] path: &ConstString,
	) -> Result<(), Error> {
		// general checks
		let doc = Document::parse(xml)?;
		let root = doc.root_element();
		if root.tag_name().name() != "root" {
			return Err(Error::WrongRootName);
		}
		if let Some(format) = root.attribute("BTCPP_format")
			&& format != "4"
		{
			return Err(Error::BtCppFormat);
		}

		// handle the attribute 'main_tree_to_execute`
		if let Some(name) = root.attribute("main_tree_to_execute") {
			registry.set_main_tree_id(name);
		}
		#[cfg(feature = "std")]
		Self::register_document_root(registry, &root, xml, path)?;
		#[cfg(not(feature = "std"))]
		Self::register_document_root(registry, &root, xml)?;
		Ok(())
	}

	#[instrument(level = Level::DEBUG, skip_all)]
	fn register_tree_nodes_model(registry: &mut BehaviorRegistry, model: &Node) -> Result<(), Error> {
		event!(Level::TRACE, "register_tree_nodes_model");
		for element in model.children() {
			match element.node_type() {
				NodeType::Root => return Err(Error::InvalidRootElement),
				NodeType::Element => {
					// an entry in the tree nodes model
					let behavior_type = element.tag_name().name();
					let mut behavior_id = behavior_type;
					for attribute in element.attributes() {
						match attribute.name() {
							"ID" => {
								behavior_id = attribute.value();
							}
							value => {
								return Err(Error::UnknownAttribute { key: value.into() });
							}
						}
					}
					for child in element.children() {
						match child.node_type() {
							NodeType::Root => return Err(Error::InvalidRootElement),
							NodeType::Element => {
								let port_type = child.tag_name().name();
								if let Some(port_name) = child.attribute(NAME)
									&& let Some(port_default) = child.attribute(DEFAULT)
								{
									let key = String::from(behavior_id) + port_name;
									let Ok(port_type) = PortDirection::try_from(port_type) else {
										return Err(Error::PortType { value: port_type.into() });
									};
									let entry = TreeNodesModelEntry {
										_port_type: port_type,
										key: port_name.into(),
										remapping: port_default.into(),
									};
									match registry.add_tree_nodes_model_entry(key.into(), entry) {
										Ok(()) => {}
										Err(err) => {
											return Err(Error::Factory {
												behavior: behavior_id.into(),
												source: err,
											});
										}
									}
								}
							}
							NodeType::PI => {
								return Err(Error::UnsupportedElement {
									tag: element.tag_name().name().into(),
								});
							}
							NodeType::Comment | NodeType::Text => {}
						}
					}
				}
				NodeType::PI => {
					return Err(Error::UnsupportedElement {
						tag: element.tag_name().name().into(),
					});
				}
				NodeType::Comment | NodeType::Text => {}
			}
		}
		Ok(())
	}

	#[instrument(level = Level::DEBUG, skip_all)]
	fn register_document_root(
		registry: &mut BehaviorRegistry,
		root: &Node,
		source: &str,
		#[cfg(feature = "std")] path: &ConstString,
	) -> Result<(), Error> {
		event!(Level::TRACE, "register_document_root");
		for element in root.children() {
			match element.node_type() {
				NodeType::Comment | NodeType::Text => {} // ignore
				NodeType::Root => return Err(Error::InvalidRootElement),
				NodeType::Element => {
					// only 'BehaviorTree' or 'TreeNodesModel' are valid
					let name = element.tag_name().name();
					match name {
						TREENODESMODEL => {
							Self::register_tree_nodes_model(registry, &element)?;
						}
						BEHAVIORTREE => {
							// check for tree ID
							if let Some(id) = element.attribute(ID) {
								// if no explicit main tree id is given, the first found id will be used for main tree
								if registry.main_tree_id().is_none() {
									registry.set_main_tree_id(id);
								}
								// let source: ConstString = element.document().input_text()[element.range()].into();
								match registry.add_tree_defintion(id, source.into(), element.range()) {
									Ok(()) => {}
									Err(err) => {
										return Err(Error::Factory {
											behavior: id.into(),
											source: err,
										});
									}
								}
							} else {
								return Err(Error::MissingId {
									tag: element.tag_name().name().into(),
								});
							}
						}
						#[cfg(feature = "std")]
						"include" => {
							let mut file_path: PathBuf;
							if let Some(path_attr) = element.attribute("path") {
								file_path = PathBuf::from(path_attr);
								if file_path.is_relative() {
									// use the given path
									file_path = PathBuf::from(path.as_ref());
									file_path.push(path_attr);
								}
							} else {
								return Err(Error::MissingPath {
									tag: element.tag_name().name().into(),
								});
							}
							match std::fs::read_to_string(&file_path) {
								Ok(xml) => {
									if let Some(cur_path) = file_path.parent() {
										let path = cur_path.to_string_lossy().into();
										Self::register_document(registry, &xml, &path)?;
									} else {
										return Err(Error::ReadFile {
											name: file_path.to_string_lossy().into(),
											cause: "no parent".into(),
										});
									}
								}
								Err(err) => {
									return Err(Error::ReadFile {
										name: file_path.to_string_lossy().into(),
										cause: err.to_string().into(),
									});
								}
							}
						}
						_ => {
							return Err(Error::UnsupportedElement {
								tag: element.tag_name().name().into(),
							});
						}
					}
				}
				NodeType::PI => {
					return Err(Error::UnsupportedElement {
						tag: element.tag_name().name().into(),
					});
				}
			}
		}
		Ok(())
	}

	#[instrument(level = Level::DEBUG, skip_all)]
	pub(crate) fn create_tree_from_definition(
		&mut self,
		name: &str,
		registry: &mut BehaviorRegistry,
		external_blackboard: Option<&Databoard>,
	) -> Result<BehaviorTreeElement, Error> {
		event!(Level::TRACE, "create_tree_from_definition");

		registry.find_tree_definition(name).map_or_else(
			|| Err(Error::DefinitionNotFound { id: name.into() }),
			|(definition, range)| {
				let doc = Document::parse(&definition[range])?;
				let data = create_data_collection_from_xml(
					registry,
					EMPTY_STR,
					&doc.root_element(),
					self.next_uid(),
					external_blackboard,
					true,
				)?;
				// for tree root "path" is empty
				let children = self.build_children(&data, &doc.root_element(), registry)?;
				if children.len() > 1 {
					return Err(Error::OneChild { behavior: name.into() });
				}
				let behaviortree = BehaviorTreeElement::create_subtree(data, children);
				Ok(behaviortree)
			},
		)
	}

	#[instrument(level = Level::DEBUG, skip_all)]
	fn build_children(
		&mut self,
		data: &BehaviorDataCollection,
		element: &Node,
		registry: &mut BehaviorRegistry,
	) -> Result<BehaviorTreeElementList, Error> {
		event!(Level::TRACE, "build_children");
		let mut children = BehaviorTreeElementList::default();
		for child in element.children() {
			match child.node_type() {
				NodeType::Comment | NodeType::Text => {} // ignore
				NodeType::Root => {
					// this should not happen
					return Err(Error::InvalidRootElement);
				}
				NodeType::Element => {
					let element = self.build_child(data, &child, registry)?;
					children.push(element);
				}
				NodeType::PI => {
					return Err(Error::UnsupportedElement {
						tag: element.tag_name().name().into(),
					});
				}
			}
		}
		Ok(children)
	}

	#[instrument(level = Level::DEBUG, skip_all)]
	fn build_child(
		&mut self,
		data: &BehaviorDataCollection,
		node: &Node,
		registry: &mut BehaviorRegistry,
	) -> Result<BehaviorTreeElement, Error> {
		event!(Level::TRACE, "build_child");
		let data =
			create_data_collection_from_xml(registry, &data.path, node, self.next_uid(), Some(&data.blackboard), false)?;
		let tree_node = match data.bhvr_desc.kind() {
			BehaviorKind::Action | BehaviorKind::Condition => {
				// A leaf uses a cloned Blackboard
				if node.has_children() {
					return Err(Error::ChildrenNotAllowed {
						behavior: data.behavior_name.into(),
					});
				}
				BehaviorTreeElement::create_leaf(data)
			}
			BehaviorKind::Control | BehaviorKind::Decorator => {
				// A node uses a cloned Blackboard
				let children = self.build_children(&data, node, registry)?;

				if data.bhvr_desc.kind() == BehaviorKind::Decorator && children.len() != 1 {
					return Err(Error::OneChild {
						behavior: node.tag_name().name().into(),
					});
				}
				BehaviorTreeElement::create_node(data, children)
			}
			BehaviorKind::SubTree => {
				if let Some(id) = node.attribute(ID) {
					match registry.find_tree_definition(id) {
						Some((definition, range)) => {
							let doc = Document::parse(&definition[range])?;
							let children = self.build_children(&data, &doc.root_element(), registry)?;
							if children.len() > 1 {
								return Err(Error::OneChild { behavior: id.into() });
							}
							BehaviorTreeElement::create_subtree(data, children)
						}
						None => {
							return Err(Error::DefinitionNotFound {
								id: data.behavior_name.into(),
							});
						}
					}
				} else {
					return Err(Error::MissingId {
						tag: node.tag_name().name().into(),
					});
				}
			}
		};
		Ok(tree_node)
	}
}
// endregion:   --- XmlParser
