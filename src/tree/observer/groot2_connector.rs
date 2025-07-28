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
use parking_lot::Mutex;
#[cfg(feature = "std")]
use tokio::{sync::mpsc, task::JoinHandle, time::Instant};
use uuid::Uuid;
use zeromq::{RepSocket, Socket, SocketRecv, SocketSend, ZmqMessage};

use crate::behavior::{BehaviorData, BehaviorState};
use crate::tree::observer::groot2_protocol::{
    Groot2ReplyHeader, Groot2RequestHeader, Groot2RequestType, Groot2TransitionInfo,
};
use crate::{ConstString, XmlCreator};

use crate::tree::BehaviorTree;
// endregion:   --- modules

const TRANSITION_SIZE: u32 = 25;

// region:      --- Groot2Connector
/// The [`Groot2Connector`] is used to create an interface between Groot2
/// and the tree executor.
///
/// The connection is via TCP and has to be established by Groot2.
/// So the connector on tree side only needs to know the port it shall listen on.
pub struct Groot2Connector<'a> {
    /// Flag for recording transitions, accessible from multiple tasks
    recording: Arc<Mutex<bool>>,
    /// A reference to the observed tree
    root: &'a BehaviorTree,
    /// The state buffer for Groot communication
    state_buffer: Arc<Mutex<BytesMut>>,
    /// The transitions buffer for Groot communication
    transitions_buffer: Arc<Mutex<VecDeque<Groot2TransitionInfo>>>,
    /// Response server
    server_handle: JoinHandle<Result<(), zeromq::ZmqError>>,
}

impl<'a> Groot2Connector<'a> {
    /// Construct a new [`Groot2Connector`].
    /// # Panics
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn new(root: &'a mut BehaviorTree, port: u16) -> Self {
        let recording = Arc::new(Mutex::new(false));
        // an empty transitions buffer
        let transitions_buffer = Arc::new(Mutex::new(VecDeque::new()));
        // a state buffer
        let tree_size = root.size() - 1; // without root
        let state_buffer = Arc::new(Mutex::new(BytesMut::zeroed((3 * tree_size) as usize)));
        // initialize state buffer
        let mut buf = state_buffer.lock();
        for i in 0..tree_size {
            let index = (3 * i) as usize;
            let bytes = (i + 1).to_be_bytes();
            buf[index] = bytes[0];
            buf[index] = bytes[1];
        }
        drop(buf);

        let id: ConstString = "groot_state".into();
        // add a callback for each tree element
        let size = Arc::new(Mutex::new(0));
        for element in root.iter_mut() {
            let size_clone = size.clone();
            let recording_clone = recording.clone();
            let transitions_buffer_clone = transitions_buffer.clone();
            let state_buffer_clone = state_buffer.clone();
            let callback = move |behavior: &BehaviorData, new_state: &mut BehaviorState| {
                let old_state = behavior.state();
                if old_state != *new_state {
                    let timestamp = Instant::now();
                    // Groot does not want a state for root
                    if behavior.uid() > 0 && *new_state != behavior.state() {
                        let state = if *new_state == BehaviorState::Idle {
                            behavior.state() as u8 + 10
                        } else {
                            *new_state as u8
                        };
                        let index = 3 * ((behavior.uid() - 1) as usize);
                        {
                            let mut buf = state_buffer_clone.lock();
                            let bytes = behavior.uid().to_be_bytes();
                            buf[index] = bytes[0];
                            buf[index] = bytes[1];
                            buf[index + 2] = state;
                        }

                        if *recording_clone.lock() {
                            #[cfg(feature = "std")]
                            #[allow(clippy::cast_possible_truncation)]
                            let timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .expect("Time went backwards")
                                .as_micros() as u64;
                            #[cfg(not(feature = "std"))]
                            let timestamp = 1753525195699631u64;
                            let info =
                                Groot2TransitionInfo::new(timestamp, behavior.uid(), *new_state);
                            let mut buf_guard = transitions_buffer_clone.lock();
                            let mut buf_size_guard = size_clone.lock();
                            if buf_guard.is_empty() {
                                *buf_size_guard = 0;
                            } else if *buf_size_guard >= TRANSITION_SIZE {
                                buf_guard.pop_front();
                            } else {
                                *buf_size_guard += 1;
                            }
                            buf_guard.push_back(info);
                            drop(buf_size_guard);
                            drop(buf_guard);
                        }
                    }
                }
            };
            element.add_pre_state_change_callback(id.clone(), callback);
        }

        // @TODO: proper error handling
        let state_buffer_clone = state_buffer.clone();
        let transitions_buffer_clone = transitions_buffer.clone();
        let tree_id = root.uuid();
        let xml = XmlCreator::groot_write_tree(root).expect("snh");
        let recording_flag = recording.clone();

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
                                reply.push_back(state_buffer_clone.lock().clone().into());
                            }
                            Groot2RequestType::FullTree => {
                                reply.push_back(xml.as_bytes().to_owned().into());
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
                                std::dbg!(&request);
                                // @TODO: todo!()
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
                                            *recording_flag.lock() = true;
                                            {
                                                let mut buf = transitions_buffer_clone.lock();
                                                // clear transition buffer
                                                buf.clear();
                                                // ensure that we can store at least TRANSITION_SIZE elements
                                                buf.reserve(TRANSITION_SIZE as usize);
                                            }
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
                                            *recording_flag.lock() = false;
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
                                let mut buf = transitions_buffer_clone.lock();
                                for info in buf.iter() {
                                    bytes.extend(Bytes::from(info));
                                }
                                // std::println!("{:?}", &bytes);
                                reply.push_back(Bytes::from(bytes));
                                buf.clear();
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

        Self {
            recording,
            root,
            state_buffer,
            transitions_buffer,
            server_handle,
        }
    }
}
// endregion:   --- Groot2Connector
