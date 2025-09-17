use crate::LogEntry;
use std::fmt::Debug;

use vstd::prelude::*;

verus!{
/// The message which Raft server used to communicate between each other
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RaftMessage<T>
where
    T: Sized + Clone + PartialEq + Eq + Debug + Default,
{
    ClientRequest {
        dest: usize,
        value: T,
    },
    BecomeLeader {
        dest: usize,
        followers: Vec<usize>,
    },
    AppendEntries {
        dest: usize,
        followers: Vec<usize>,
    },
    AppendEntriesRequest {
        src: usize,
        dest: usize,
        term: usize,
        prev_index: usize,
        prev_term: usize,
        commit_index: usize,
        entries: Vec<LogEntry<T>>,
    },
    AppendEntriesResponse {
        src: usize,
        dest: usize,
        term: usize,
        success: bool,
        match_index: usize,
    },
    TimeOut {
        dest: usize,
        followers: Vec<usize>,
    },
    RequestVoteRequest {
        src: usize,
        dest: usize,
        term: usize,
        last_log_index: usize,
        last_log_term: usize,
    },
    RequestVoteResponse {
        src: usize,
        dest: usize,
        term: usize,
        vote_granted: bool,
    },
}
}
