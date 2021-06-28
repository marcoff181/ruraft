//!
//! # Raft Model
//! `raftmodel` aims to provide the rust implementations
//! of logic model for the raft consensus algorithm.
//!
//! # The big picture
//! `raftmodel` is a crate to provide a pure logic model for the raft algorithm.
//! It aims to stay as close as possible to its formal TLA+ specification ([raft.tla](https://github.com/ongardie/raft.tla/blob/master/raft.tla)).
//! # Quick Start
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
