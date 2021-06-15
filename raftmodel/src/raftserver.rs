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
    match_index: Option<Vec<usize>>,
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
            match_index: Option::None,
        }
    }

    pub fn handle_message(&mut self, msg: RaftMessage<T>) -> Vec<RaftMessage<T>> {
        match msg {
            RaftMessage::ClientRequest { dest, value } => self.handle_client_request(dest, value),
            RaftMessage::BecomeLeader { dest, followers } => {
                self.handle_become_leader(dest, followers)
            }
            RaftMessage::AppendEntries { dest, followers } => {
                self.handle_append_entries(dest, followers)
            }
            RaftMessage::AppendEntriesRequest {
                src,
                dest,
                term,
                prev_index,
                prev_term,
                entries,
            } => {
                self.handle_append_entries_request(src, dest, term, prev_index, prev_term, entries)
            }
            RaftMessage::AppendEntriesResponse {
                src,
                dest,
                term,
                success,
                match_index,
            } => self.handle_append_entries_response(src, dest, term, success, match_index),
            _ => {
                println!("Nothing");
                vec![]
            }
        }
    }

    fn handle_client_request(&mut self, dest: u32, value: T) -> Vec<RaftMessage<T>> {
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
        self.next_index = Some(vec![self.log.len() as usize; followers.len() + 1]);
        return self.handle_append_entries(dest, followers);
        //vec![]
    }

    fn handle_append_entries(&mut self, dest: u32, followers: Vec<u32>) -> Vec<RaftMessage<T>> {
        if self.state != ServerStates::Leader {
            return vec![];
        }
        let mut msgs = vec![];
        for i in followers {
            if i == dest {
                continue;
            }
            let nxt = (self.next_index.as_ref().unwrap())[i as usize];
            let prev_index = nxt - 1;
            let prev_term = self.log[prev_index].term;
            let entries = self.log[nxt as usize..].to_vec();
            // dbg!(prev_index);
            // dbg!(prev_term);
            msgs.push(RaftMessage::AppendEntriesRequest {
                src: dest,
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
        let elen: i128 = entries.len() as i128;
        dbg!(prev_index);

        let success = append_entries(&mut self.log, prev_index, prev_term, entries);

        msgs.push(RaftMessage::AppendEntriesResponse {
            src: dest,
            dest: src,
            term: self.current_term,
            success,
            match_index: prev_index + elen,
        });

        msgs
    }

    fn handle_append_entries_response(
        &mut self,
        src: u32,
        dest: u32,
        term: i128,
        success: bool,
        match_index: i128,
    ) -> Vec<RaftMessage<T>> {
        let mut msgs = vec![];
        if term != self.current_term {
            return msgs;
        }

        msgs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    fn run_message<T>(initial_message: RaftMessage<T>, servers: &mut Vec<RaftServer<T>>)
    where
        T: Sized + Clone + PartialEq + Eq + Debug + Default,
    {
        let mut messages = VecDeque::new();
        messages.push_back(initial_message);
        while let Some(msg) = messages.pop_front() {
            let dest = match msg {
                RaftMessage::ClientRequest { dest, .. }
                | RaftMessage::BecomeLeader { dest, .. }
                | RaftMessage::AppendEntries { dest, .. }
                | RaftMessage::AppendEntriesRequest { dest, .. }
                | RaftMessage::AppendEntriesResponse { dest, .. } => dest,
            };
            let server = &mut servers[dest as usize];
            let mut responses = server.handle_message(msg);
            messages.append(&mut responses.into_iter().collect());
        }
    }

    #[test]
    fn test_replicate() {
        let mut servers = vec![
            RaftServer::new(vec![LogEntry { term: 1, item: "x" }]),
            RaftServer::new(vec![]),
        ];

        run_message(
            RaftMessage::BecomeLeader {
                dest: 0,
                followers: vec![1],
            },
            &mut servers,
        );

        run_message(
            RaftMessage::AppendEntries {
                dest: 0,
                followers: vec![1],
            },
            &mut servers,
        );

        assert_eq!(servers[0].log, servers[1].log);
    }
}
