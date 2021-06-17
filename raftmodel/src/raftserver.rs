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
    current_term: usize,
    next_index: Option<Vec<usize>>,
    match_index: Option<Vec<usize>>,
    commit_index: usize,
}

impl<T> RaftServer<T>
where
    T: Sized + Clone + PartialEq + Eq + Debug + Default,
{
    pub fn new(log: Vec<LogEntry<T>>) -> RaftServer<T> {
        RaftServer {
            log: log,
            state: ServerStates::Follower,
            current_term: 1,
            next_index: Option::None,
            match_index: Option::None,
            commit_index: 0,
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

    fn handle_client_request(&mut self, dest: usize, value: T) -> Vec<RaftMessage<T>> {
        if self.state != ServerStates::Leader {
            return vec![];
        }
        let mut entries = vec![LogEntry {
            term: self.current_term,
            item: value,
        }];
        let prev_index = self.log.len() - 1;
        let prev_term = self.log[prev_index].term;
        append_entries(&mut self.log, prev_index, prev_term, entries);
        vec![]
    }

    fn handle_become_leader(&mut self, dest: usize, followers: Vec<usize>) -> Vec<RaftMessage<T>> {
        println!("{} become Leader", dest);
        self.state = ServerStates::Leader;
        self.next_index = Some(vec![self.log.len(); followers.len() + 2]);
        self.match_index = Some(vec![0; followers.len() + 2]);
        return self.handle_append_entries(dest, followers);
    }

    fn handle_append_entries(&mut self, dest: usize, followers: Vec<usize>) -> Vec<RaftMessage<T>> {
        if self.state != ServerStates::Leader {
            return vec![];
        }
        let mut msgs = vec![];
        for follower in followers {
            if follower == dest {
                continue;
            }
            let next = (self.next_index.as_ref().unwrap())[follower];
            let prev_index = next - 1;
            // dbg!(prev_index);
            // dbg!(self.log.clone());
            // dbg!(self.next_index.clone());
            let prev_term = if prev_index == 0 {
                0
            } else {
                self.log[prev_index].term
            };
            let entries = self.log[next..].to_vec();
            msgs.push(RaftMessage::AppendEntriesRequest {
                src: dest,
                dest: follower,
                term: self.current_term,
                prev_index,
                prev_term,
                entries,
            });
        }
        msgs
    }

    fn handle_append_entries_request(
        &mut self,
        src: usize,
        dest: usize,
        term: usize,
        prev_index: usize,
        prev_term: usize,
        entries: Vec<LogEntry<T>>,
    ) -> Vec<RaftMessage<T>> {
        let mut msgs = vec![];
        let elen = entries.len();

        let success = append_entries(&mut self.log, prev_index, prev_term, entries);

        msgs.push(RaftMessage::AppendEntriesResponse {
            src: dest,
            dest: src,
            term: self.current_term,
            success,
            match_index: self.log.len() - 1,
        });

        msgs
    }

    fn handle_append_entries_response(
        &mut self,
        src: usize,
        dest: usize,
        term: usize,
        success: bool,
        match_index: usize,
    ) -> Vec<RaftMessage<T>> {
        let mut msgs = vec![];
        if term != self.current_term {
            return msgs;
        }
        let next_index = self.next_index.as_mut().unwrap();

        if !success {
            next_index[src] = next_index[src] - 1;
            let mut responses = self.handle_append_entries(dest, vec![src]);
            msgs.append(&mut responses);
        } else {
            next_index[src] = match_index + 1;
            if match_index > self.match_index.as_mut().unwrap()[src] {
                self.match_index.as_mut().unwrap()[src] = match_index;
            }
            self.advance_commit_index(dest);
        }

        msgs
    }

    fn advance_commit_index(&mut self, dest: usize) {
        let match_index = self.match_index.as_mut().unwrap();
        match_index.sort_unstable();
        let mid = (match_index.len() + 1) / 2 as usize;
        let max_agree_index = match_index[mid];
        // dbg!(max_agree_index);
        if self.log[max_agree_index].term == self.current_term {
            self.commit_index = max_agree_index;
        }
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
            RaftServer::new(vec![]),
            RaftServer::new(vec![LogEntry::default(), LogEntry { term: 1, item: "x" }]),
            RaftServer::new(vec![LogEntry::default()]),
            RaftServer::new(vec![LogEntry::default()]),
        ];

        run_message(
            RaftMessage::BecomeLeader {
                dest: 1,
                followers: vec![2, 3],
            },
            &mut servers,
        );

        run_message(
            RaftMessage::AppendEntries {
                dest: 1,
                followers: vec![2, 3],
            },
            &mut servers,
        );

        assert_eq!(servers[1].log, servers[2].log);
    }

    fn make_log(terms: Vec<usize>) -> Vec<LogEntry<String>> {
        let mut result: Vec<LogEntry<String>> = vec![LogEntry::default()];
        for x in terms {
            result.push(LogEntry {
                term: x,
                item: "a".to_string(),
            });
        }
        result
    }

    #[test]
    fn test_figure_6() {
        let mut servers = vec![
            RaftServer::new(vec![LogEntry::default()]),
            RaftServer::new(make_log(vec![1, 1, 1, 2, 3, 3, 3, 3])),
            RaftServer::new(make_log(vec![1, 1, 1, 2, 3])),
            RaftServer::new(make_log(vec![1, 1, 1, 2, 3, 3, 3, 3])),
            RaftServer::new(make_log(vec![1, 1])),
            RaftServer::new(make_log(vec![1, 1, 1, 2, 3, 3, 3])),
        ];

        for server in &mut servers {
            server.current_term = 3;
        }
        dbg!("aaa");
        run_message(
            RaftMessage::BecomeLeader {
                dest: 1,
                followers: (2..6).collect(),
            },
            &mut servers,
        );

        run_message(
            RaftMessage::AppendEntries {
                dest: 1,
                followers: (2..6).collect(),
            },
            &mut servers,
        );
        dbg!(&servers[1].log);
        dbg!(&servers[2].log);
        assert!(servers.iter().skip(1).all(|x| { x.log == servers[1].log }));
    }
}
