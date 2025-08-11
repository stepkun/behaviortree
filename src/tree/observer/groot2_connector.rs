// Copyright © 2025 Stephan Kunz
#![allow(unused)]

//! [`Groot2Connector`] implementation.
//!

extern crate std;

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;
// region:      --- modules
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use bytes::{BufMut, Bytes, BytesMut};
use core::default;
use core::fmt::Display;
use core::time::Duration;
use spin::Mutex;
#[cfg(feature = "std")]
use tokio::{sync::mpsc, task::JoinHandle, time::Instant};
use uuid::Uuid;
use zeromq::{RepSocket, Socket, SocketRecv, SocketSend, ZmqMessage};

use crate::behavior::{BehaviorData, BehaviorState};
use crate::tree::observer::groot2_protocol::{
    Groot2ReplyHeader, Groot2RequestHeader, Groot2RequestType, Groot2TransitionInfo,
};
use crate::tree::tree::BehaviorTreeMessage;
use crate::{ConstString, SHOULD_NOT_HAPPEN, XmlCreator};

use crate::tree::tree::BehaviorTree;
// endregion:   --- modules

/// Predefined size of the behavior state transition buffer.
/// This amount will be buffered between sends.
/// If there are more state transitions happening, the eldest will be dropped.
const TRANSITION_SIZE: u32 = 100;

/// constants
pub const GROOT_STATE: &str = "groot_state";

// region:      --- GrootCallback
/// Attach the Groot2 communication callbacks to a [`BehaviorTree`].
/// # Panics
/// - if an unknown message from Groot2 arrives
pub fn attach_groot_callback(tree: &mut BehaviorTree, shared: Arc<Mutex<Groot2ConnectorData>>) {
    let id: ConstString = GROOT_STATE.into();
    // add a callback for each tree element
    let size = Arc::new(Mutex::new(0));
    let shared = shared;
    for element in tree.iter_mut() {
        let shared_clone = shared.clone();
        // the callback
        let callback = move |behavior: &BehaviorData, new_state: &mut BehaviorState| {
            if behavior.state() != *new_state {
                // Groot does not need a state for root
                if behavior.uid() != 0 {
                    let state = if *new_state == BehaviorState::Idle {
                        behavior.state() as u8 + 10
                    } else {
                        *new_state as u8
                    };
                    let mut shared_guard = shared_clone.lock();
                    let uid = behavior.uid().to_le_bytes();
                    let index = 3 * ((behavior.uid() - 1) as usize);
                    shared_guard.state_buffer[index] = uid[0];
                    shared_guard.state_buffer[index + 1] = uid[1];
                    shared_guard.state_buffer[index + 2] = state;

                    if shared_guard.recording {
                        #[cfg(feature = "std")]
                        #[allow(clippy::cast_possible_truncation)]
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .expect("Time went backwards")
                            .as_micros() as u64;
                        #[cfg(not(feature = "std"))]
                        let timestamp = 1753525195699631u64;
                        let info = Groot2TransitionInfo::new(timestamp, behavior.uid(), *new_state);
                        if shared_guard.transitions_buffer.is_empty() {
                            shared_guard.transitions = 0;
                        } else if shared_guard.transitions >= TRANSITION_SIZE {
                            shared_guard.transitions_buffer.pop_front();
                        } else {
                            shared_guard.transitions += 1;
                        }
                        shared_guard.transitions_buffer.push_back(info);
                    }
                    drop(shared_guard);
                }
            }
        };
        element.add_pre_state_change_callback(id.clone(), callback);
    }
}
// endregion:   --- GrootCallback

// region:      --- Groot2Connector
/// The [`Groot2Connector`] is used to create an interface between Groot2
/// and the tree executor.
///
/// The connection is via TCP and has to be established by Groot2.
/// So the connector on tree side only needs to know the port it shall listen on.
pub struct Groot2Connector {
    /// The sender to send messages to tree
    tx: mpsc::Sender<BehaviorTreeMessage>,
    /// Shared data across multiple tasks (callbacks)
    shared: Arc<Mutex<Groot2ConnectorData>>,
    /// Response server
    server_handle: JoinHandle<Result<(), zeromq::ZmqError>>,
}

/// The shared data among multiple [`BehaviorTreeElement`]s.
pub struct Groot2ConnectorData {
    /// The state buffer for Groot communication
    state_buffer: BytesMut,
    /// Flag for recording transitions, accessible from multiple tasks
    recording: bool,
    /// Current size of the transition buffer
    transitions: u32,
    /// The transitions buffer for Groot communication
    transitions_buffer: VecDeque<Groot2TransitionInfo>,
}

impl Groot2Connector {
    /// Construct a new [`Groot2Connector`].
    /// # Panics
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn new(tree: &mut BehaviorTree, port: u16) -> Self {
        // an empty transitions buffer
        let transitions_buffer = VecDeque::new();
        // a state buffer
        let tree_size = tree.size() - 1; // without root
        let mut state_buffer = BytesMut::zeroed((3 * tree_size) as usize);
        // initialize state buffer
        for i in 0..tree_size {
            let index = (3 * i) as usize;
            let bytes = (i + 1).to_be_bytes();
            state_buffer[index] = bytes[0];
            state_buffer[index] = bytes[1];
        }

        let shared = Arc::new(Mutex::new(Groot2ConnectorData {
            state_buffer,
            recording: false,
            transitions: 0,
            transitions_buffer,
        }));

        // @TODO: proper error handling
        let shared_clone = shared.clone();
        let tree_id = tree.uuid();
        let xml = XmlCreator::groot_write_tree(tree).expect(SHOULD_NOT_HAPPEN);
        let sender = tree.sender();

        #[cfg(feature = "std")]
        let server_handle = tokio::spawn(async move {
            let server_address = String::from("tcp://0.0.0.0:") + &port.to_string();
            let mut server_socket = zeromq::RepSocket::new();
            server_socket.bind(&server_address).await?;

            loop {
                let mut request = server_socket.recv().await?;
                // std::dbg!(&request);
                if let Some(bytes) = request.get(0) {
                    // std::dbg!(bytes);
                    if let Ok(header) = Groot2RequestHeader::try_from(bytes) {
                        let rq_type = header.rq_type();
                        let reply_header = Groot2ReplyHeader::new(header, tree_id);
                        let mut reply = ZmqMessage::from(Bytes::from(&reply_header));
                        match rq_type {
                            // most requests will be "State"
                            Groot2RequestType::State => {
                                // std::println!("{:?}", buffer.lock());
                                reply.push_back(shared_clone.lock().state_buffer.clone().into());
                            }
                            Groot2RequestType::FullTree => {
                                sender
                                    .send(BehaviorTreeMessage::AddGrootCallback(
                                        shared_clone.clone(),
                                    ))
                                    .await;
                                reply.push_back(xml.clone());
                            }
                            Groot2RequestType::BlackBoard => {
                                std::dbg!(&request);
                                todo!()
                            }
                            Groot2RequestType::HookInsert => {
                                std::dbg!(&request);
                                todo!()
                            }
                            Groot2RequestType::HookRemove => {
                                std::dbg!(&request);
                                todo!()
                            }
                            Groot2RequestType::HooksDump => {
                                std::dbg!(&request);
                                todo!()
                            }
                            Groot2RequestType::RemoveAllHooks => {
                                sender.send(BehaviorTreeMessage::RemoveAllGrootHooks).await;
                            }
                            Groot2RequestType::DisableAllHooks => {
                                std::dbg!(&request);
                                todo!()
                            }
                            Groot2RequestType::BreakpointReached => {
                                std::dbg!(&request);
                                todo!()
                            }
                            Groot2RequestType::BreakpointUnlock => {
                                std::dbg!(&request);
                                todo!()
                            }
                            Groot2RequestType::ToggleRecording => {
                                if let Some(command) = request.get(1) {
                                    let cmd = command.to_vec();
                                    match &cmd[..] {
                                        b"start" => {
                                            // activate transition recording
                                            let mut shared_guard = shared_clone.lock();
                                            shared_guard.recording = true;
                                            // clear transition buffer
                                            shared_guard.transitions_buffer.clear();
                                            // ensure that we can store at least TRANSITION_SIZE elements
                                            shared_guard
                                                .transitions_buffer
                                                .reserve(TRANSITION_SIZE as usize);
                                            drop(shared_guard);
                                            // return the microseconds since 01.01.1970
                                            #[cfg(feature = "std")]
                                            #[allow(clippy::cast_possible_truncation)]
                                            let timestamp = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .expect("Time went backwards")
                                                .as_micros()
                                                as u64;
                                            #[cfg(not(feature = "std"))]
                                            let timestamp = 1753525195699631u64;
                                            reply.push_back(Bytes::from(timestamp.to_string()));
                                        }
                                        b"stop" => {
                                            // de-activate transition recording
                                            shared_clone.lock().recording = false;
                                        }
                                        _ => {
                                            // this will only happen if there is some new Groot feature
                                            #[cfg(feature = "std")]
                                            std::dbg!(&command);
                                            todo!()
                                        }
                                    }
                                } else {
                                    todo!()
                                }
                            }
                            Groot2RequestType::GetTransitions => {
                                // @TODO: send transition buffer
                                let mut bytes =
                                    BytesMut::with_capacity((TRANSITION_SIZE * 9) as usize);
                                let mut shared_guard = shared_clone.lock();
                                for info in &shared_guard.transitions_buffer {
                                    bytes.extend(Bytes::from(info));
                                }
                                // std::println!("{:?}", &bytes);
                                reply.push_back(Bytes::from(bytes));
                                shared_guard.transitions_buffer.clear();
                            }
                            Groot2RequestType::Undefined => {
                                std::dbg!(&request);
                                todo!()
                            }
                        }

                        // std::dbg!(&reply);
                        server_socket.send(reply).await?;
                    } else {
                        std::dbg!(&request);
                        todo!()
                    }
                } else {
                    todo!()
                }
            }
            Ok(())
        });
        #[cfg(not(feature = "std"))]
        {
            todo!()
        }
        Self {
            #[cfg(feature = "std")]
            tx: tree.sender(),
            shared,
            server_handle,
        }
    }
}
// endregion:   --- Groot2Connector
