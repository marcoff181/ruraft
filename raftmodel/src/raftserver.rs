use crate::{append_entries, LogEntry, RaftMessage};
use std::default::Default;
use std::fmt::Debug;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ServerStates {
    Leader,
    Candidate,
    Follower,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaftServer<T>
where
    T: Sized + Clone + PartialEq + Eq + Debug + Default,
{
    log: Vec<LogEntry<T>>,
    state: ServerStates,
    current_term: i128,
    next_index: Option<Vec<usize>>,
}

impl<T> RaftServer<T>
where
    T: Sized + Clone + PartialEq + Eq + Debug + Default,
{
    pub fn new(log: Vec<LogEntry<T>>) -> RaftServer<T> {
        RaftServer {
            log: log,
            state: ServerStates::Follower,
            current_term: 0,
            next_index: Option::None,
        }
    }

    pub fn handle_message(&mut self, msg: RaftMessage<T>) -> Vec<RaftMessage<T>> {
        match msg {
            RaftMessage::ClientRequest { value: value } => self.handle_client_request(value),
            RaftMessage::BecomeLeader {
                dest: dest,
                followers: followers,
            } => self.handle_become_leader(dest, followers),
            RaftMessage::AppendEntries {
                src: src,
                followers: followers,
            } => self.handle_append_entries(src, followers),
            RaftMessage::AppendEntriesRequest {
                src: src,
                dest: dest,
                term: term,
                prev_index: prev_index,
                prev_term: prev_term,
                entries: entries,
            } => {
                self.handle_append_entries_request(src, dest, term, prev_index, prev_term, entries)
            }
            _ => {
                println!("Nothing");
                vec![]
            }
        }
    }

    fn handle_client_request(&mut self, value: T) -> Vec<RaftMessage<T>> {
        if self.state != ServerStates::Leader {
            return vec![];
        }
        let mut entries = vec![LogEntry {
            term: 0,
            item: value,
        }];
        let prev_index = self.log.len() - 1;
        let prev_term: i128 = self.log[prev_index].term;
        append_entries(&mut self.log, prev_index as i128, prev_term, entries);
        vec![]
    }

    fn handle_become_leader(&mut self, dest: u32, followers: Vec<u32>) -> Vec<RaftMessage<T>> {
        println!("{} become Leader", dest);
        self.state = ServerStates::Leader;
        self.next_index = Some(vec![self.log.len() as usize; followers.len()]);
        vec![]
    }

    fn handle_append_entries(&mut self, src: u32, followers: Vec<u32>) -> Vec<RaftMessage<T>> {
        if self.state != ServerStates::Leader {
            return vec![];
        }
        let mut msgs = vec![];
        for i in followers {
            if i == src {
                continue;
            }
            let nxt = (self.next_index.as_ref().unwrap())[i as usize];
            let prev_index = nxt - 1;
            let prev_term = self.log[prev_index].term;
            let entries = self.log[nxt as usize..].to_vec();
            msgs.push(RaftMessage::AppendEntriesRequest {
                src,
                dest: i,
                term: self.current_term,
                prev_index: prev_index as i128,
                prev_term,
                entries,
            });
        }
        msgs
    }

    fn handle_append_entries_request(
        &mut self,
        src: u32,
        dest: u32,
        term: i128,
        prev_index: i128,
        prev_term: i128,
        entries: Vec<LogEntry<T>>,
    ) -> Vec<RaftMessage<T>> {
        let mut msgs = vec![];
        let success = append_entries(&mut self.log, prev_index, prev_term, entries);

        msgs.push(RaftMessage::AppendEntriesResponse {
            src: dest,
            dest: src,
            term: self.current_term,
            success,
        });

        msgs
    }

    fn handle_append_entries_response(
        src: u32,
        dest: u32,
        term: i128,
        success: bool,
    ) -> Vec<RaftMessage<T>> {
        let mut msgs = vec![];
        if term != self.current_term {
            return msgs;
        }

        msgs
    }
}
