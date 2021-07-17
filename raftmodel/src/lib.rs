//!
//! # Raft Model
//! `raftmodel` aims to provide the rust implementations
//! of logic model for the raft consensus algorithm.
//!
//! # The big picture
//! `raftmodel` is a crate to provide a pure logic model for the raft algorithm.
//! It aims to strictly follow raft formal TLA+ specification ([raft.tla](https://github.com/ongardie/raft.tla/blob/master/raft.tla)).
//!
//! To use `raftmodel`, user interacts with raft by calling `handle_message` method in `RaftServer`. The input of `handle_message`
//! is `RaftMessage` and the output is `Vec<RaftMessage>`.
//! For example, a client can use the following code to add a new entry to the log
//! of the leader and replicate it to the followers.
//! ```ignore
//! let response = leader.handle_message(            
//!            RaftMessage::ClientRequest {
//!                dest: 1,
//!                value: "x",
//!            },
//!     );
//! ```
//!
//! However, this crate only implements the pure logic of the raft algorithm. In order for it to work in the real environment, user
//! needs to implement the I/O in order to pass the messages between servers across the network and also store some information to the
//! persistent storage such as disks.
//! # Quick Start
//! To demonstrate how to use this crate, we define the mock up `run_message` function in the following code. It is used to simulate the
//! raft messages being passed between servers in the network which drives the log entries replicated from the leader to the followers.
//! ```
//! use crate::raftmodel::*;
//! use std::collections::{VecDeque, HashSet};
//! use std::fmt::Debug;
//! fn run_message<T>(initial_message: RaftMessage<T>, servers: &mut Vec<RaftServer<T>>)
//! where
//!     T: Sized + Clone + PartialEq + Eq + Debug + Default,
//! {
//!     let mut messages = VecDeque::new();
//!     messages.push_back(initial_message);
//!     while let Some(msg) = messages.pop_front() {
//!         let dest = match msg {
//!             RaftMessage::ClientRequest { dest, .. }
//!             | RaftMessage::BecomeLeader { dest, .. }
//!             | RaftMessage::AppendEntries { dest, .. }
//!             | RaftMessage::AppendEntriesRequest { dest, .. }
//!             | RaftMessage::AppendEntriesResponse { dest, .. }
//!             | RaftMessage::RequestVoteRequest { dest, .. }
//!             | RaftMessage::RequestVoteResponse { dest, .. }
//!             | RaftMessage::TimeOut { dest, .. } => dest,
//!         };
//!         let server = &mut servers[dest as usize];
//!         let responses = server.handle_message(msg);
//!         messages.append(&mut responses.into_iter().collect());
//!     }
//! }
//!
//!
//!
//! let log = create_empty_log();
//!
//! let mut servers = vec![
//!    RaftServer::new(log.clone()),
//!    RaftServer::new(log.clone()),
//!    RaftServer::new(log.clone()),
//!    RaftServer::new(log.clone()),
//!    RaftServer::new(log.clone()),
//!    RaftServer::new(log.clone()),
//! ];
//!
//! // Let server 1 time out to become a candidate. It should win the election with all votes
//! run_message(
//!     RaftMessage::TimeOut {
//!         dest: 1,
//!         followers: (2..6).collect(),
//!     },
//!     &mut servers,
//! );
//! assert_eq!(*servers[1].server_state(), ServerState::Leader);
//!
//! // Client append a new entry to the leader's log
//! run_message(
//!    RaftMessage::ClientRequest{
//!        dest:1,
//!        value: "x".to_string(),
//!    },
//!    &mut servers,
//! );
//!
//! // The first AppendEntries will update leader commit_index
//! run_message(
//!     RaftMessage::AppendEntries {
//!         dest: 1,
//!         followers: (2..6).collect(),
//!     },
//!     &mut servers,
//! );
//!
//! // The second AppendEntries will update all followers commit_index
//! run_message(
//!     RaftMessage::AppendEntries {
//!         dest: 1,
//!         followers: (2..6).collect(),
//!     },
//!     &mut servers,
//! );
//!
//! assert!(servers.iter().skip(1).all(|x| { servers[1].log() == x.log() }));
//!
//!
//! ```

mod raftmodel;

pub use crate::raftmodel::*;
