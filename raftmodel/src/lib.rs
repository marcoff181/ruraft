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
//!
//! fn make_log(terms: Vec<usize>) -> Vec<LogEntry<String>> {
//!    let mut result: Vec<LogEntry<String>> = vec![LogEntry::default()];
//!    for x in terms {
//!        result.push(LogEntry {
//!            term: x,
//!            item: "a".to_string(),
//!        });
//!    }
//!    result
//!}
//!
//! let mut servers = vec![
//!    RaftServer::new(vec![LogEntry::default()]),
//!    RaftServer::new(make_log(vec![1, 1, 1, 2, 3, 3, 3, 3])),
//!    RaftServer::new(make_log(vec![1, 1, 1, 2, 3])),
//!    RaftServer::new(make_log(vec![1, 1, 1, 2, 3, 3, 3, 3])),
//!    RaftServer::new(make_log(vec![1, 1])),
//!    RaftServer::new(make_log(vec![1, 1, 1, 2, 3, 3, 3])),
//! ];
//! ```

mod raftmodel;

pub use crate::raftmodel::*;
