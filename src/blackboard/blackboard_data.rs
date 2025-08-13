// Copyright © 2025 Stephan Kunz

//! Blackboard for behaviortree

// region:      --- modules
use crate::{ConstString, SHOULD_NOT_HAPPEN};
use alloc::{
	borrow::ToOwned,
	collections::btree_map::BTreeMap,
	format,
	string::{String, ToString},
	sync::Arc,
};
use core::{
	any::{Any, TypeId},
	fmt::Debug,
	ops::{Deref, DerefMut},
	str::FromStr,
};
use tinyscript::{Environment, Error as ScriptingError, execution::ScriptingValue};

use super::{BlackboardInterface, error::Error};
// endregion:   --- modules

// region:      --- BlackboardData
/// `BlackboardData` are a key value store capable of storing any value.
#[derive(Debug, Default)]
pub struct BlackboardData {
	storage: BTreeMap<ConstString, Entry>,
}

impl BlackboardInterface for BlackboardData {
	fn contains(&self, key: &str) -> bool {
		self.storage.contains_key(key)
	}

	fn delete<T>(&mut self, key: &str) -> Result<T, Error>
	where
		T: Any + Clone + FromStr + ToString + Send + Sync,
	{
		if let Some(old_entry) = self.storage.get(key) {
			let e = &*old_entry.0.0;
			let e = e as &dyn Any;
			let e = e.downcast_ref::<T>().cloned();
			if let Some(old) = e {
				self.storage.remove(key);
				Ok(old)
			} else {
				Err(Error::WrongType(key.into()))
			}
		} else {
			Err(Error::NotFound(key.into()))
		}
	}

	fn get<T>(&self, key: &str) -> Result<T, Error>
	where
		T: Any + Clone + FromStr + ToString + Send + Sync,
	{
		self.storage.get(key).map_or_else(
			|| Err(Error::NotFound(key.into())),
			|entry| {
				let en = &*entry.0.0;
				en.downcast_ref::<T>().map_or_else(
					|| {
						en.downcast_ref::<String>().map_or_else(
							|| {
								// maybe it is a value set by scripting
								self.get_env(key).map_or_else(
									|_| Err(Error::WrongType(key.into())),
									|val| {
										let s = match val {
											ScriptingValue::Nil() => unreachable!(),
											ScriptingValue::Boolean(b) => b.to_string(),
											ScriptingValue::Float64(f) => f.to_string(),
											ScriptingValue::Int64(i) => i.to_string(),
											ScriptingValue::String(s) => s,
										};
										T::from_str(&s).map_or_else(
											|_| {
												Err(Error::ParsePortValue(
													key.into(),
													format!("{:?}", TypeId::of::<T>()).into(),
												))
											},
											|val| Ok(val),
										)
									},
								)
							},
							|s| {
								T::from_str(s).map_or_else(
									|_| Err(Error::ParsePortValue(key.into(), format!("{:?}", TypeId::of::<T>()).into())),
									|val| Ok(val),
								)
							},
						)
					},
					|value| Ok(value.clone()),
				)
			},
		)
	}

	fn get_sequence_id(&self, key: &str) -> Result<usize, Error> {
		self.storage
			.get(key)
			.map_or_else(|| Err(Error::NotFound(key.into())), |entry| Ok(entry.0.1))
	}

	fn get_entry(&self, key: &str) -> Option<Entry> {
		self.storage.get(key).map_or_else(
			|| None,
			|entry| {
				let e = entry.0.clone();
				Some(Entry(e))
			},
		)
	}

	fn set<T>(&mut self, key: &str, value: T) -> Result<Option<T>, Error>
	where
		T: Any + Clone + FromStr + ToString + Send + Sync,
	{
		if let Some(old_entry) = self.storage.get(key) {
			let new_id = if old_entry.0.1 < usize::MAX {
				old_entry.0.1 + 1
			} else {
				usize::MIN
			};
			let e = &*old_entry.0.0;
			let e = e as &dyn Any;
			let e = e.downcast_ref::<T>().cloned();
			if e.is_some() {
				let entry = Entry((Arc::new(value), new_id));
				self.storage.insert(key.into(), entry);
				Ok(e)
			} else {
				Err(Error::WrongType(key.into()))
			}
		} else {
			// sequence_id starts with 1, 0 is used as non existent
			let entry = Entry((Arc::new(value), usize::MIN + 1));
			self.storage.insert(key.into(), entry);
			Ok(None)
		}
	}
}

impl Environment for BlackboardData {
	fn define_env(&mut self, key: &str, value: ScriptingValue) -> Result<(), ScriptingError> {
		if self.contains(key) {
			self.set_env(key, value)
		} else {
			match value {
				ScriptingValue::Nil() => unreachable!(),
				ScriptingValue::Boolean(b) => {
					self.set(key, b).expect(SHOULD_NOT_HAPPEN);
				}
				ScriptingValue::Float64(f) => {
					self.set(key, f).expect(SHOULD_NOT_HAPPEN);
				}
				ScriptingValue::Int64(i) => {
					self.set(key, i).expect(SHOULD_NOT_HAPPEN);
				}
				ScriptingValue::String(s) => {
					self.set(key, s).expect(SHOULD_NOT_HAPPEN);
				}
			}
			Ok(())
		}
	}

	fn get_env(&self, name: &str) -> Result<ScriptingValue, ScriptingError> {
		self.get_entry(name).map_or_else(
			|| Err(ScriptingError::GlobalNotDefined(name.into())),
			|entry| {
				// let entry = **(entry);
				let type_id = (**entry).type_id();
				if type_id == TypeId::of::<String>() {
					let s = entry
						.downcast_ref::<String>()
						.expect(SHOULD_NOT_HAPPEN);
					Ok(ScriptingValue::String(s.to_owned()))
				} else if type_id == TypeId::of::<f64>() {
					let f = entry
						.downcast_ref::<f64>()
						.expect(SHOULD_NOT_HAPPEN);
					Ok(ScriptingValue::Float64(f.to_owned()))
				} else if type_id == TypeId::of::<f32>() {
					let f = entry
						.downcast_ref::<f32>()
						.expect(SHOULD_NOT_HAPPEN);
					Ok(ScriptingValue::Float64(f64::from(f.to_owned())))
				} else if type_id == TypeId::of::<i64>() {
					let i = entry
						.downcast_ref::<i64>()
						.expect(SHOULD_NOT_HAPPEN);
					Ok(ScriptingValue::Int64(i.to_owned()))
				} else if type_id == TypeId::of::<i32>() {
					let i = entry
						.downcast_ref::<i32>()
						.expect(SHOULD_NOT_HAPPEN);
					Ok(ScriptingValue::Int64(i64::from(i.to_owned())))
				} else if type_id == TypeId::of::<u32>() {
					let i = entry
						.downcast_ref::<u32>()
						.expect(SHOULD_NOT_HAPPEN);
					Ok(ScriptingValue::Int64(i64::from(i.to_owned())))
				} else if type_id == TypeId::of::<i16>() {
					let i = entry
						.downcast_ref::<i16>()
						.expect(SHOULD_NOT_HAPPEN);
					Ok(ScriptingValue::Int64(i64::from(i.to_owned())))
				} else if type_id == TypeId::of::<u16>() {
					let i = entry
						.downcast_ref::<u16>()
						.expect(SHOULD_NOT_HAPPEN);
					Ok(ScriptingValue::Int64(i64::from(i.to_owned())))
				} else if type_id == TypeId::of::<u8>() {
					let i = entry
						.downcast_ref::<u8>()
						.expect(SHOULD_NOT_HAPPEN);
					Ok(ScriptingValue::Int64(i64::from(i.to_owned())))
				} else if type_id == TypeId::of::<i8>() {
					let i = entry
						.downcast_ref::<i8>()
						.expect(SHOULD_NOT_HAPPEN);
					Ok(ScriptingValue::Int64(i64::from(i.to_owned())))
				} else {
					Err(ScriptingError::GlobalHasUnknownType(name.into()))
				}
			},
		)
	}

	#[allow(clippy::cast_possible_truncation)]
	#[allow(clippy::cast_sign_loss)]
	fn set_env(&mut self, name: &str, value: ScriptingValue) -> Result<(), ScriptingError> {
		let entry_type_id = if let Some(entry) = self.get_entry(name) {
			let inner_entry = &entry;
			(*(inner_entry.0.0)).type_id()
		} else {
			return Err(ScriptingError::GlobalNotDefined(name.into()));
		};
		match value {
			ScriptingValue::Nil() => unreachable!(),
			ScriptingValue::Boolean(b) => {
				if TypeId::of::<bool>() == entry_type_id {
					self.set(name, b).expect(SHOULD_NOT_HAPPEN);
				} else {
					return Err(ScriptingError::GlobalWrongType(name.into()));
				}
			}
			ScriptingValue::Float64(f) => {
				if TypeId::of::<f64>() == entry_type_id {
					self.set(name, f).expect(SHOULD_NOT_HAPPEN);
				} else if TypeId::of::<f32>() == entry_type_id {
					if f > f64::from(f32::MAX) || f < f64::from(f32::MIN) {
						return Err(ScriptingError::GlobalExceedsLimits(name.into()));
					}
					self.set(name, f as f32).expect(SHOULD_NOT_HAPPEN);
				} else {
					return Err(ScriptingError::GlobalWrongType(name.into()));
				}
			}
			ScriptingValue::Int64(i) => {
				if TypeId::of::<i64>() == entry_type_id {
					self.set(name, i).expect(SHOULD_NOT_HAPPEN);
				} else if TypeId::of::<i32>() == entry_type_id {
					if i > i64::from(i32::MAX) || i < i64::from(i32::MIN) {
						return Err(ScriptingError::GlobalExceedsLimits(name.into()));
					}
					self.set(name, i as i32).expect(SHOULD_NOT_HAPPEN);
				} else if TypeId::of::<u32>() == entry_type_id {
					if i > i64::from(u32::MAX) || i < i64::from(u32::MIN) {
						return Err(ScriptingError::GlobalExceedsLimits(name.into()));
					}
					self.set(name, i as u32).expect(SHOULD_NOT_HAPPEN);
				} else if TypeId::of::<i16>() == entry_type_id {
					if i > i64::from(i16::MAX) || i < i64::from(i16::MIN) {
						return Err(ScriptingError::GlobalExceedsLimits(name.into()));
					}
					self.set(name, i as i16).expect(SHOULD_NOT_HAPPEN);
				} else if TypeId::of::<u16>() == entry_type_id {
					if i > i64::from(u16::MAX) || i < i64::from(u16::MIN) {
						return Err(ScriptingError::GlobalExceedsLimits(name.into()));
					}
					self.set(name, i as u16).expect(SHOULD_NOT_HAPPEN);
				} else if TypeId::of::<i8>() == entry_type_id {
					if i > i64::from(i8::MAX) || i < i64::from(i8::MIN) {
						return Err(ScriptingError::GlobalExceedsLimits(name.into()));
					}
					self.set(name, i as i8).expect(SHOULD_NOT_HAPPEN);
				} else if TypeId::of::<u8>() == entry_type_id {
					if i > i64::from(u8::MAX) || i < i64::from(u8::MIN) {
						return Err(ScriptingError::GlobalExceedsLimits(name.into()));
					}
					self.set(name, i as u8).expect(SHOULD_NOT_HAPPEN);
				} else {
					return Err(ScriptingError::GlobalWrongType(name.into()));
				}
			}
			ScriptingValue::String(s) => {
				if TypeId::of::<String>() == entry_type_id {
					self.set(name, s).expect(SHOULD_NOT_HAPPEN);
				} else {
					return Err(ScriptingError::GlobalWrongType(name.into()));
				}
			}
		}
		Ok(())
	}
}
// endregion:   --- BlackboardData

// region:      --- Entry
pub struct Entry((Arc<dyn Any + Send + Sync>, usize));

impl Debug for Entry {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{:?}", self.0)?;
		Ok(())
	}
}

impl Deref for Entry {
	type Target = Arc<dyn Any + Send + Sync>;

	fn deref(&self) -> &Self::Target {
		&self.0.0
	}
}

impl DerefMut for Entry {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0.0
	}
}
// endregion:   --- Entry
