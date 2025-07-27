// Copyright © 2025 Stephan Kunz
#![allow(unused)]

//! [`Groot2Connector`] implementation.
//!

extern crate std;

use alloc::borrow::ToOwned;
// region:      --- modules
use alloc::string::ToString;
use bytes::{Bytes, BytesMut, BufMut};
use core::default;
use core::fmt::Display;
use parking_lot::Mutex;
use uuid::Uuid;

use crate::ConstString;
use crate::behavior::BehaviorState;

use crate::tree::BehaviorTree;
// endregion:   --- modules

// region:      --- Groot2RequestType
/// The type of request to and from Groot2.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum Groot2RequestType {
	/// Undefined request
	#[default]
	Undefined = 0,

	/// Request the entire tree definition as XML
	FullTree = b'T',
	/// Request the status of all the nodes
	State = b'S',
	/// retrieve the values in a set of blackboards
	BlackBoard = b'B',

	/// Groot requests the insertion of a hook
	HookInsert = b'I',
	/// Groot requests to remove a hook
	HookRemove = b'R',
	/// receive the existing hooks in JSON format
	HooksDump = b'D',
	/// Remove all hooks. To be done before disconnecting Groot
	RemoveAllHooks = b'A',
	/// Disable all hooks
	DisableAllHooks = b'X',

	/// Notify Groot that we reached a breakpoint
	BreakpointReached = b'N',
	/// Groot will unlock a breakpoint
	BreakpointUnlock = b'U',

	/// Toggle recording
	ToggleRecording = b'r',
	/// Get all status transitions during recording
	GetTransitions = b't',
}

/// Version of the protocol
const PROTOCOL_ID: u8 = 2;

/// Constant strings for Display etc.
const UNDEFINED: &str = "undefined";
const FULLTREE: &str = "full_tree";
const STATUS: &str = "status";
const BLACKBOARD: &str = "blackboard";
const HOOK_INSERT: &str = "hook_insert";
const HOOK_REMOVE: &str = "hook_remove";
const BREAKPOINT_REACHED: &str = "breakpoint_reached";
const BREAKPOINT_UNLOCK: &str = "breakpoint_unlock";
const REMOVE_ALL_HOOKS: &str = "hooks_remove_all";
const HOOKS_DUMP: &str = "hooks_dump";
const DISABLE_ALL_HOOKS: &str = "disable_hooks";
const TOGGLE_RECORDING: &str = "toggle_recording";
const GET_TRANSITIONS: &str = "get_transitions";

impl Display for Groot2RequestType {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		let text = match self {
			Self::Undefined => UNDEFINED,
			Self::FullTree => FULLTREE,
			Self::State => STATUS,
			Self::BlackBoard => BLACKBOARD,
			Self::HookInsert => HOOK_INSERT,
			Self::HookRemove => HOOK_REMOVE,
			Self::HooksDump => HOOKS_DUMP,
			Self::RemoveAllHooks => REMOVE_ALL_HOOKS,
			Self::DisableAllHooks => DISABLE_ALL_HOOKS,
			Self::BreakpointReached => BREAKPOINT_REACHED,
			Self::BreakpointUnlock => BREAKPOINT_UNLOCK,
			Self::ToggleRecording => TOGGLE_RECORDING,
			Self::GetTransitions => GET_TRANSITIONS,
		};
		write!(f, "{text}")
	}
}

impl From<&Groot2RequestType> for u8 {
	fn from(value: &Groot2RequestType) -> Self {
		*value as Self
	}
}

impl TryFrom<u8> for Groot2RequestType {
	type Error = crate::tree::error::Error;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			b'T' => Ok(Self::FullTree),
			b'S' => Ok(Self::State),
			b'B' => Ok(Self::BlackBoard),
			b'I' => Ok(Self::HookInsert),
			b'R' => Ok(Self::HookRemove),
			b'D' => Ok(Self::HooksDump),
			b'A' => Ok(Self::RemoveAllHooks),
			b'X' => Ok(Self::DisableAllHooks),
			b'N' => Ok(Self::BreakpointReached),
			b'U' => Ok(Self::BreakpointUnlock),
			b'r' => Ok(Self::ToggleRecording),
			b't' => Ok(Self::GetTransitions),
			_ => Err(Self::Error::InvalidRequestType(value)),
		}
	}
}
// endregion:   --- Groot2RequestType

// region:      --- Groot2RequestHeader
/// Request header for communication with Groot.
#[derive(Debug)]
#[repr(C)]
pub struct Groot2RequestHeader {
	protocol_id: u8,
	rq_type: Groot2RequestType,
	uid: [u8; 4],
}

impl From<&Groot2RequestHeader> for Bytes {
	fn from(value: &Groot2RequestHeader) -> Self {
		let mut bytes = BytesMut::zeroed(6);
		bytes[0] = value.protocol_id;
		bytes[1] = u8::from(&value.rq_type);
		// let uid = value.uid.to_be_bytes();
		bytes[2] = value.uid[0];
		bytes[3] = value.uid[1];
		bytes[4] = value.uid[2];
		bytes[5] = value.uid[3];
		bytes.into()
	}
}

impl TryFrom<&Bytes> for Groot2RequestHeader {
	type Error = crate::tree::error::Error;

	fn try_from(value: &Bytes) -> Result<Self, Self::Error> {
		let uid: [u8; 4] = [value[2], value[3], value[4], value[5]];
		let protocol_id = value[0];
		let rq_type = Groot2RequestType::try_from(value[1])?;
		Ok(Self {
			protocol_id,
			rq_type,
			uid,
		})
	}
}

impl Groot2RequestHeader {
	/// Get the request type
	pub(crate) const fn rq_type(&self) -> Groot2RequestType {
		self.rq_type
	}
}
// endregion:   --- Groot2RequestHeader

// region:      --- Groot2ReplyHeader
/// Reply header for communication with Groot.
#[derive(Debug)]
#[repr(C)]
pub struct Groot2ReplyHeader {
	rq_header: Groot2RequestHeader,
	tree_id: Uuid,
}

impl From<&Groot2ReplyHeader> for Bytes {
	fn from(value: &Groot2ReplyHeader) -> Self {
		let mut bytes = BytesMut::with_capacity(22);
		bytes.extend(Self::from(&value.rq_header));
		bytes.extend(value.tree_id.as_bytes());
		bytes.into()
	}
}

impl Groot2ReplyHeader {
	/// Create a reply header
	#[must_use]
	pub(crate) const fn new(rq_header: Groot2RequestHeader, tree_id: Uuid) -> Self {
		Self { rq_header, tree_id }
	}
}
// endregion:   --- Groot2ReplyHeader

// region:      --- Groot2TransitionInfo
/// Structure holds transition informations.
pub struct Groot2TransitionInfo {
	/// microseconds since epoch
	timestamp: u64,
	/// behaviors uid
	uid: u16,
	/// The [`BehaviorState `] as u8
	state: u8,
}

impl From<&Groot2TransitionInfo> for Bytes {
	fn from(value: &Groot2TransitionInfo) -> Self {
		let mut bytes = BytesMut::with_capacity(9);
		let timestamp = value.timestamp.to_ne_bytes();
		bytes.extend_from_slice(&timestamp[..6]);
		let uid = value.uid.to_ne_bytes();
		bytes.extend_from_slice(&uid[..]);
		bytes.put_u8(value.state);		
		bytes.into()
	}
}

impl Groot2TransitionInfo {
	/// Create a transition info
	#[must_use]
	pub const fn new(timestamp: u64, uid: u16, state: BehaviorState) -> Self {
		Self {
			timestamp,
			uid,
			state: state as u8,
		}
	}
}
// endregion:   --- Groot2TransitionInfo

// region:      --- Groot2Hook
#[repr(C)]
enum Position {
	Pre = 0,
	Post = 1,
}

#[repr(C)]
enum Mode {
	Breakpoint = 0,
	Replace = 1,
}

/// Hook for Breakpoints in debugging with Groot2.
pub struct Groot2Hook {
	/// used to enable/disable the breakpoint, default is `true`
	enabled: bool,
	/// Differentiation between pre- and post-position of hook
	position: Position,
	/// behaviors uid
	uid: u16,
	/// Differentiation between Breakpoint and Replacement
	/// @TODO: interactive breakpoints are unblocked using `unlockBreakpoint()`
	mode: Mode,
	/// used by interactive breakpoints to wait for unlocking @ TODO:
	wakeup: ConditionVariable,
	/// A Mutex @TODO:
	mutex: Mutex<u32>,
	/// set to true to unlock an interactive breakpoint, default is `false`
	ready: bool,
	/// once finished self-destroy, default is `false`
	remove_when_done: bool,
	/// result to be returned, default is `Skipped`
	desired_status: BehaviorState,
}
// endregion:   --- Groot2Hook

// region:      --- ConditionVariable
/// @TODO:
pub struct ConditionVariable {}
// endregion:   --- ConditionVariable

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn serde() {
		let header = Groot2RequestHeader {
			uid: [1, 2, 3, 4],
			protocol_id: 2,
			rq_type: Groot2RequestType::FullTree,
		};
		let bytes = Bytes::from(&header);
		let deserialized = Groot2RequestHeader::try_from(&bytes).expect("snh");
		assert_eq!(deserialized.protocol_id, header.protocol_id);
		assert_eq!(deserialized.rq_type, header.rq_type);
		assert_eq!(deserialized.uid, header.uid);
	}
}
