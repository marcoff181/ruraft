use crate::LogEntry;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RaftMessage<T>
where
    T: Sized + Clone + PartialEq + Eq + Debug + Default,
{
    ClientRequest {
        dest: u32,
        value: T,
    },
    BecomeLeader {
        dest: u32,
        followers: Vec<u32>,
    },
    AppendEntries {
        dest: u32,
        followers: Vec<u32>,
    },
    AppendEntriesRequest {
        src: u32,
        dest: u32,
        term: i128,
        prev_index: i128,
        prev_term: i128,
        entries: Vec<LogEntry<T>>,
    },
    AppendEntriesResponse {
        src: u32,
        dest: u32,
        term: i128,
        success: bool,
        match_index: i128,
    },
}
