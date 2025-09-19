// Copyright © 2025 Stephan Kunz
//! Built-In behaviors of [`behaviortree`](crate).

use crate::{
	BehaviorState, ConstString,
	behavior::{BehaviorDataCollection, BehaviorTickCallback, behavior_description::BehaviorDescription},
	port::error::Error,
};
use alloc::{
	borrow::ToOwned,
	boxed::Box,
	string::{String, ToString},
	vec::Vec,
};
use core::{
	any::{Any, TypeId},
	fmt::Debug,
	str::FromStr,
};
use databoard::{
	Databoard, DataboardPtr, EntryReadGuard, EntryWriteGuard, Remappings, check_board_pointer, strip_board_pointer,
};
use tinyscript::{Environment, ScriptingValue};

// region:		--- helpers
/// Removes enclosing brackets `{}` from a str if there are any,
/// otherwise returns the unchanged str.
#[must_use]
fn strip_curly_brackets(key: &str) -> &str {
	let key = key.strip_prefix('{').unwrap_or(key);
	key.strip_suffix('}').unwrap_or(key)
}
// endregion:	--- helpers

// region:      --- BehaviorData
/// Structure for implementing behaviors.
#[derive(Default)]
pub struct BehaviorData {
	/// UID of the behavior within the [`BehaviorTree`](crate::tree::BehaviorTree).
	/// 65536 behaviors in a [`BehaviorTree`](crate::tree::BehaviorTree) should be sufficient.
	/// The ordering of the uid is following the creation order by the [`XmlParser`](crate::factory::xml_parser::XmlParser).
	/// This should end up in a depth first ordering.
	uid: u16,
	/// Current state of the behavior.
	state: BehaviorState,
	/// List of internal [`Remappings`] including
	/// direct assigned values to a `Port`, e.g. default values.
	remappings: Remappings,
	/// Reference to the [`Blackboard`] for the element.
	blackboard: DataboardPtr,
	/// List of pre state change callbacks with an identifier.
	/// These callbacks can be used for observation of the [`BehaviorTreeElement`] and
	/// for manipulation of the resulting [`BehaviorState`] of a tick.
	pre_state_change_hooks: Vec<(ConstString, Box<BehaviorTickCallback>)>,
	/// Description of the Behavior.
	description: BehaviorDescription,
}

impl BehaviorData {
	/// Constructor
	#[must_use]
	pub(crate) fn new(data: &BehaviorDataCollection) -> Self {
		Self {
			uid: data.uid,
			state: BehaviorState::default(),
			remappings: data.remappings.clone(),
			blackboard: data.blackboard.clone(),
			pre_state_change_hooks: Vec::default(),
			description: data.bhvr_desc.clone(),
		}
	}

	/// Returns `true` if the `key` is available, otherwise `false`.
	#[must_use]
	pub fn contains(&self, key: &str) -> bool {
		let key = strip_curly_brackets(key);
		let key = self.remappings.remap(key);
		self.blackboard().contains_key(&key)
	}

	/// Delete an entry of type `T` from Blackboard.
	/// # Errors
	/// - if entry is not found
	pub fn delete<T>(&mut self, key: &str) -> Result<T, Error>
	where
		T: Any + Clone + Debug + FromStr + ToString + Send + Sync,
	{
		let remapped_key = self.remappings.remap(key);
		let board_key = match check_board_pointer(&remapped_key) {
			Ok(board_pointer) => board_pointer,
			Err(original_key) => original_key,
		};
		Ok(self.blackboard.delete::<T>(board_key)?)
	}

	/// Get a value of type `T` from Blackboard.
	/// # Errors
	/// - if value is not found
	#[allow(clippy::option_if_let_else)]
	#[allow(clippy::single_match_else)]
	pub fn get<T>(&self, key: &str) -> Result<T, Error>
	where
		T: Any + Clone + Debug + FromStr + ToString + Send + Sync,
	{
		// #[cfg(feature = "std")]
		// extern crate std;

		if let Some(remapped) = self.remappings.find(key) {
			// std::dbg!("remapped");
			match strip_board_pointer(&remapped) {
				Some(remapped_key) => match self.blackboard.entry(remapped_key) {
					Ok(entry) => {
						let en = &*entry.read();
						let data = en.data().as_ref();
						if let Some(val) = data.downcast_ref::<T>() {
							// std::dbg!("remapped1");
							Ok(val.clone())
						} else if let Some(val) = data.downcast_ref::<String>() {
							// std::dbg!("remapped2");
							match T::from_str(val) {
								Ok(res) => Ok(res),
								Err(_) => Err(Error::CouldNotConvert(remapped_key.into())),
							}
						} else {
							// std::dbg!("remapped3");
							// maybe it is a value set by scripting
							self.get_env(remapped_key).map_or_else(
								|_| Err(Error::NotFound(remapped.clone())),
								|val| {
									let s = match val {
										ScriptingValue::Nil() => unreachable!(),
										ScriptingValue::Boolean(b) => b.to_string(),
										ScriptingValue::Float64(f) => f.to_string(),
										ScriptingValue::Int64(i) => i.to_string(),
										ScriptingValue::String(s) => s,
									};
									T::from_str(&s)
										.map_or_else(|_| Err(Error::CouldNotConvert(remapped_key.into())), |val| Ok(val))
								},
							)
						}
					}
					Err(err) => {
						// std::dbg!("remapped4");
						match err {
							databoard::Error::Assignment { key: _, value } => T::from_str(&value)
								.map_or_else(|_| Err(Error::CouldNotConvert(remapped_key.into())), |val| Ok(val)),
							_ => Err(err.into()),
						}
					}
				},
				None => {
					// std::dbg!("remapped5");
					match T::from_str(&remapped) {
						Ok(res) => Ok(res),
						Err(_err) => Err(Error::CouldNotConvert(remapped)),
					}
				}
			}
		} else {
			// std::dbg!("NOT remapped");
			match check_board_pointer(key) {
				Ok(board_ptr) => match self.blackboard.get::<T>(board_ptr) {
					Ok(value) => Ok(value),
					Err(err) => {
						let entry = self.blackboard.entry(key)?;
						let en = &*entry.read();
						if let Some(val) = en.data().downcast_ref::<String>() {
							match T::from_str(val) {
								Ok(res) => Ok(res),
								Err(_) => Err(Error::CouldNotConvert(key.into())),
							}
						} else {
							Err(err.into())
						}
					}
				},
				Err(original_key) => match self.blackboard.get::<T>(original_key) {
					Ok(value) => Ok(value),
					Err(err) => {
						let entry = self.blackboard.entry(key)?;
						let en = &*entry.read();
						if let Some(val) = en.data().downcast_ref::<String>() {
							match T::from_str(val) {
								Ok(res) => Ok(res),
								Err(_) => Err(Error::CouldNotConvert(key.into())),
							}
						} else {
							Err(err.into())
						}
					}
				},
			}
		}
	}

	/// Returns a reference to value of type `T` from Blackboard.
	/// # Errors
	/// - if value is not found
	#[allow(clippy::option_if_let_else)]
	#[allow(clippy::single_match_else)]
	pub fn get_ref<T>(&self, key: &str) -> Result<EntryReadGuard<T>, Error>
	where
		T: Any + Clone + Debug + FromStr + ToString + Send + Sync,
	{
		let remapped_key = self.remappings.remap(key);
		let board_key = match check_board_pointer(&remapped_key) {
			Ok(board_pointer) => board_pointer,
			Err(original_key) => original_key,
		};
		Ok(self.blackboard.get_ref::<T>(board_key)?)
	}

	/// Returns a mutable reference to value of type `T` from Blackboard.
	/// # Errors
	/// - if value is not found
	#[allow(clippy::option_if_let_else)]
	#[allow(clippy::single_match_else)]
	pub fn get_mut_ref<T>(&self, key: &str) -> Result<EntryWriteGuard<T>, Error>
	where
		T: Any + Clone + Debug + FromStr + ToString + Send + Sync,
	{
		let remapped_key = self.remappings.remap(key);
		let board_key = match check_board_pointer(&remapped_key) {
			Ok(board_pointer) => board_pointer,
			Err(original_key) => original_key,
		};
		Ok(self.blackboard.get_mut_ref::<T>(board_key)?)
	}

	/// Set a value of type `T` into Blackboard.
	/// Returns old value if any.
	/// # Errors
	/// - if value can not be set
	pub fn set<T>(&mut self, key: &str, value: T) -> Result<Option<T>, Error>
	where
		T: Any + Clone + Debug + FromStr + ToString + Send + Sync,
	{
		let remapped_key = self.remappings.remap(key);
		let board_key = match check_board_pointer(&remapped_key) {
			Ok(board_pointer) => board_pointer,
			Err(original_key) => original_key,
		};
		Ok(self.blackboard.set::<T>(board_key, value)?)
	}

	/// Get the sequence ID of a Blackboard entry.
	/// # Errors
	/// - if key is not found in blackboard
	#[inline]
	pub fn sequence_id(&self, key: &str) -> Result<usize, databoard::Error> {
		self.blackboard.sequence_id(key)
	}

	/// Method to access the blackboard.
	#[must_use]
	pub fn blackboard(&self) -> &Databoard {
		&self.blackboard
	}

	/// Method to get the desription.
	#[must_use]
	pub const fn description(&self) -> &BehaviorDescription {
		&self.description
	}

	/// Method to get the desription mutable.
	#[must_use]
	pub const fn description_mut(&mut self) -> &mut BehaviorDescription {
		&mut self.description
	}

	/// Method to get the uid.
	#[must_use]
	pub const fn uid(&self) -> u16 {
		self.uid
	}

	/// Method to get the state.
	#[must_use]
	pub const fn state(&self) -> BehaviorState {
		self.state
	}

	/// Method to set the state.
	pub fn set_state(&mut self, state: BehaviorState) {
		if state != self.state {
			// Callback before setting state
			let mut state = state;
			for (_, callback) in &self.pre_state_change_hooks {
				callback(self, &mut state);
			}
			self.state = state;
		}
	}

	/// Add a pre state change callback with the given name.
	/// The name is not unique, which is important when removing callback.
	pub fn add_pre_state_change_callback<T>(&mut self, name: ConstString, callback: T)
	where
		T: Fn(&Self, &mut BehaviorState) + Send + Sync + 'static,
	{
		self.pre_state_change_hooks
			.push((name, Box::new(callback)));
	}

	/// Remove any pre state change callback with the given name.
	pub fn remove_pre_state_change_callback(&mut self, name: &ConstString) {
		// first collect all subscriber with that name ...
		let mut indices = Vec::new();
		for (index, (cb_name, _)) in self.pre_state_change_hooks.iter().enumerate() {
			if cb_name == name {
				indices.push(index);
			}
		}
		// ... then remove them from vec
		for index in indices {
			let _ = self.pre_state_change_hooks.remove(index);
		}
	}

	pub(crate) const fn remappings(&self) -> &Remappings {
		&self.remappings
	}
}
// endregion:	--- BehaviorData

// region:		--- impl Environment
impl Environment for BehaviorData {
	fn define_env(&mut self, key: &str, value: ScriptingValue) -> Result<(), tinyscript::environment::Error> {
		if self.contains(key) {
			self.set_env(key, value)
		} else {
			match value {
				ScriptingValue::Nil() => unreachable!(),
				ScriptingValue::Boolean(b) => match self.set(key, b) {
					Ok(_) => {}
					Err(cause) => {
						return Err(tinyscript::environment::Error::EnvVarSet {
							name: key.into(),
							cause: cause.to_string().into(),
						});
					}
				},
				ScriptingValue::Float64(f) => match self.set(key, f) {
					Ok(_) => {}
					Err(cause) => {
						return Err(tinyscript::environment::Error::EnvVarSet {
							name: key.into(),
							cause: cause.to_string().into(),
						});
					}
				},
				ScriptingValue::Int64(i) => match self.set(key, i) {
					Ok(_) => {}
					Err(cause) => {
						return Err(tinyscript::environment::Error::EnvVarSet {
							name: key.into(),
							cause: cause.to_string().into(),
						});
					}
				},
				ScriptingValue::String(s) => match self.set(key, s) {
					Ok(_) => {}
					Err(cause) => {
						return Err(tinyscript::environment::Error::EnvVarSet {
							name: key.into(),
							cause: cause.to_string().into(),
						});
					}
				},
			}
			Ok(())
		}
	}

	#[allow(clippy::too_many_lines)]
	fn get_env(&self, name: &str) -> Result<ScriptingValue, tinyscript::environment::Error> {
		self.blackboard().entry(name).map_or_else(
			|_| Err(tinyscript::environment::Error::EnvVarNotDefined { name: name.into() }),
			|entry| {
				let entry = entry.read();
				let type_id = (**entry).as_ref().type_id();
				if type_id == TypeId::of::<String>() {
					let s =
						entry
							.downcast_ref::<String>()
							.ok_or_else(|| tinyscript::environment::Error::EnvVarTypeCast {
								name: name.into(),
								var_type: "String".into(),
							})?;
					Ok(ScriptingValue::String(s.to_owned()))
				} else if type_id == TypeId::of::<f64>() {
					let f = entry
						.downcast_ref::<f64>()
						.ok_or_else(|| tinyscript::environment::Error::EnvVarTypeCast {
							name: name.into(),
							var_type: "f64".into(),
						})?;
					Ok(ScriptingValue::Float64(f.to_owned()))
				} else if type_id == TypeId::of::<f32>() {
					let f = entry
						.downcast_ref::<f32>()
						.ok_or_else(|| tinyscript::environment::Error::EnvVarTypeCast {
							name: name.into(),
							var_type: "f32".into(),
						})?;
					Ok(ScriptingValue::Float64(f64::from(f.to_owned())))
				} else if type_id == TypeId::of::<i64>() {
					let i = entry
						.downcast_ref::<i64>()
						.ok_or_else(|| tinyscript::environment::Error::EnvVarTypeCast {
							name: name.into(),
							var_type: "i64".into(),
						})?;
					Ok(ScriptingValue::Int64(i.to_owned()))
				} else if type_id == TypeId::of::<i32>() {
					let i = entry
						.downcast_ref::<i32>()
						.ok_or_else(|| tinyscript::environment::Error::EnvVarTypeCast {
							name: name.into(),
							var_type: "i32".into(),
						})?;
					Ok(ScriptingValue::Int64(i64::from(i.to_owned())))
				} else if type_id == TypeId::of::<u32>() {
					let i = entry
						.downcast_ref::<u32>()
						.ok_or_else(|| tinyscript::environment::Error::EnvVarTypeCast {
							name: name.into(),
							var_type: "u32".into(),
						})?;
					Ok(ScriptingValue::Int64(i64::from(i.to_owned())))
				} else if type_id == TypeId::of::<i16>() {
					let i = entry
						.downcast_ref::<i16>()
						.ok_or_else(|| tinyscript::environment::Error::EnvVarTypeCast {
							name: name.into(),
							var_type: "i16".into(),
						})?;
					Ok(ScriptingValue::Int64(i64::from(i.to_owned())))
				} else if type_id == TypeId::of::<u16>() {
					let i = entry
						.downcast_ref::<u16>()
						.ok_or_else(|| tinyscript::environment::Error::EnvVarTypeCast {
							name: name.into(),
							var_type: "u16".into(),
						})?;
					Ok(ScriptingValue::Int64(i64::from(i.to_owned())))
				} else if type_id == TypeId::of::<u8>() {
					let i = entry
						.downcast_ref::<u8>()
						.ok_or_else(|| tinyscript::environment::Error::EnvVarTypeCast {
							name: name.into(),
							var_type: "u8".into(),
						})?;
					Ok(ScriptingValue::Int64(i64::from(i.to_owned())))
				} else if type_id == TypeId::of::<i8>() {
					let i = entry
						.downcast_ref::<i8>()
						.ok_or_else(|| tinyscript::environment::Error::EnvVarTypeCast {
							name: name.into(),
							var_type: "i8".into(),
						})?;
					Ok(ScriptingValue::Int64(i64::from(i.to_owned())))
				} else {
					Err(tinyscript::environment::Error::EnvVarUnknownType { name: name.into() })
				}
			},
		)
	}

	#[allow(clippy::too_many_lines)]
	#[allow(clippy::cast_possible_truncation)]
	#[allow(clippy::cast_sign_loss)]
	fn set_env(&mut self, name: &str, value: ScriptingValue) -> Result<(), tinyscript::environment::Error> {
		let entry_type_id = match self.blackboard().entry(name) {
			Ok(entry) => {
				let en = entry.read();
				let data = en.as_ref();
				data.type_id()
			}
			Err(_) => {
				return Err(tinyscript::environment::Error::EnvVarNotDefined { name: name.into() });
			}
		};
		match value {
			ScriptingValue::Nil() => unreachable!(),
			ScriptingValue::Boolean(b) => {
				if TypeId::of::<bool>() == entry_type_id {
					match self.set(name, b) {
						Ok(_) => {}
						Err(cause) => {
							return Err(tinyscript::environment::Error::EnvVarSet {
								name: name.into(),
								cause: cause.to_string().into(),
							});
						}
					}
				} else {
					return Err(tinyscript::environment::Error::EnvVarWrongType { name: name.into() });
				}
			}
			ScriptingValue::Float64(f) => {
				if TypeId::of::<f64>() == entry_type_id {
					match self.set(name, f) {
						Ok(_) => {}
						Err(cause) => {
							return Err(tinyscript::environment::Error::EnvVarSet {
								name: name.into(),
								cause: cause.to_string().into(),
							});
						}
					}
				} else if TypeId::of::<f32>() == entry_type_id {
					if f > f64::from(f32::MAX) || f < f64::from(f32::MIN) {
						return Err(tinyscript::environment::Error::EnvVarExceedsLimits { name: name.into() });
					}
					match self.set(name, f) {
						Ok(_) => {}
						Err(cause) => {
							return Err(tinyscript::environment::Error::EnvVarSet {
								name: name.into(),
								cause: cause.to_string().into(),
							});
						}
					}
				} else {
					return Err(tinyscript::environment::Error::EnvVarWrongType { name: name.into() });
				}
			}
			ScriptingValue::Int64(i) => {
				if TypeId::of::<i64>() == entry_type_id {
					match self.set(name, i) {
						Ok(_) => {}
						Err(cause) => {
							return Err(tinyscript::environment::Error::EnvVarSet {
								name: name.into(),
								cause: cause.to_string().into(),
							});
						}
					}
				} else if TypeId::of::<i32>() == entry_type_id {
					if i > i64::from(i32::MAX) || i < i64::from(i32::MIN) {
						return Err(tinyscript::environment::Error::EnvVarExceedsLimits { name: name.into() });
					}
					match self.set(name, i as i32) {
						Ok(_) => {}
						Err(cause) => {
							return Err(tinyscript::environment::Error::EnvVarSet {
								name: name.into(),
								cause: cause.to_string().into(),
							});
						}
					}
				} else if TypeId::of::<u32>() == entry_type_id {
					if i > i64::from(u32::MAX) || i < i64::from(u32::MIN) {
						return Err(tinyscript::environment::Error::EnvVarExceedsLimits { name: name.into() });
					}
					match self.set(name, i as u32) {
						Ok(_) => {}
						Err(cause) => {
							return Err(tinyscript::environment::Error::EnvVarSet {
								name: name.into(),
								cause: cause.to_string().into(),
							});
						}
					}
				} else if TypeId::of::<i16>() == entry_type_id {
					if i > i64::from(i16::MAX) || i < i64::from(i16::MIN) {
						return Err(tinyscript::environment::Error::EnvVarExceedsLimits { name: name.into() });
					}
					match self.set(name, i as i16) {
						Ok(_) => {}
						Err(cause) => {
							return Err(tinyscript::environment::Error::EnvVarSet {
								name: name.into(),
								cause: cause.to_string().into(),
							});
						}
					}
				} else if TypeId::of::<u16>() == entry_type_id {
					if i > i64::from(u16::MAX) || i < i64::from(u16::MIN) {
						return Err(tinyscript::environment::Error::EnvVarExceedsLimits { name: name.into() });
					}
					match self.set(name, i as u16) {
						Ok(_) => {}
						Err(cause) => {
							return Err(tinyscript::environment::Error::EnvVarSet {
								name: name.into(),
								cause: cause.to_string().into(),
							});
						}
					}
				} else if TypeId::of::<i8>() == entry_type_id {
					if i > i64::from(i8::MAX) || i < i64::from(i8::MIN) {
						return Err(tinyscript::environment::Error::EnvVarExceedsLimits { name: name.into() });
					}
					match self.set(name, i as i8) {
						Ok(_) => {}
						Err(cause) => {
							return Err(tinyscript::environment::Error::EnvVarSet {
								name: name.into(),
								cause: cause.to_string().into(),
							});
						}
					}
				} else if TypeId::of::<u8>() == entry_type_id {
					if i > i64::from(u8::MAX) || i < i64::from(u8::MIN) {
						return Err(tinyscript::environment::Error::EnvVarExceedsLimits { name: name.into() });
					}
					match self.set(name, i as u8) {
						Ok(_) => {}
						Err(cause) => {
							return Err(tinyscript::environment::Error::EnvVarSet {
								name: name.into(),
								cause: cause.to_string().into(),
							});
						}
					}
				} else {
					return Err(tinyscript::environment::Error::EnvVarWrongType { name: name.into() });
				}
			}
			ScriptingValue::String(s) => {
				if TypeId::of::<String>() == entry_type_id {
					match self.set(name, s) {
						Ok(_) => {}
						Err(cause) => {
							return Err(tinyscript::environment::Error::EnvVarSet {
								name: name.into(),
								cause: cause.to_string().into(),
							});
						}
					}
				} else {
					return Err(tinyscript::environment::Error::EnvVarWrongType { name: name.into() });
				}
			}
		}
		Ok(())
	}
}
// endregion:	--- impl Environment
