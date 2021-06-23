use crate::{append_entries, LogEntry, RaftMessage};
use std::collections::HashSet;
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
    // The following attributes are all per server
    log: Vec<LogEntry<T>>,
    state: ServerStates,
    current_term: usize,
    voted_for: usize,
    commit_index: usize,
    last_applied: usize,

    // The following attributes are used only on candidates
    votes_responded: Option<HashSet<usize>>,
    votes_granted: Option<HashSet<usize>>,
    followers: Option<Vec<usize>>,

    // The following attributes are used only on leaders
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
            current_term: 1,
            voted_for: 0,
            commit_index: 0,
            last_applied: 0,
            votes_responded: Option::None,
            votes_granted: Option::None,
            followers: Option::None,
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
                commit_index,
                entries,
            } => {
                self.update_term(term);
                self.handle_append_entries_request(
                    src,
                    dest,
                    term,
                    prev_index,
                    prev_term,
                    commit_index,
                    entries,
                )
            }
            RaftMessage::AppendEntriesResponse {
                src,
                dest,
                term,
                success,
                match_index,
            } => {
                if term < self.current_term {
                    return vec![];
                }
                self.update_term(term);
                self.handle_append_entries_response(src, dest, term, success, match_index)
            }
            RaftMessage::RequestVoteRequest {
                src,
                dest,
                term,
                last_log_index,
                last_log_term,
            } => {
                self.update_term(term);
                self.handle_request_vote_request(src, dest, term, last_log_index, last_log_term)
            }
            RaftMessage::RequestVoteResponse {
                src,
                dest,
                term,
                vote_granted,
            } => {
                if term < self.current_term {
                    return vec![];
                }
                self.update_term(term);
                self.handle_request_vote_response(src, dest, term, vote_granted)
            }
            RaftMessage::TimeOut { dest, followers } => self.handle_time_out(dest, followers),
        }
    }

    fn handle_client_request(&mut self, dest: usize, value: T) -> Vec<RaftMessage<T>> {
        if self.state != ServerStates::Leader {
            return vec![];
        }
        let entries = vec![LogEntry {
            term: self.current_term,
            item: value,
        }];
        let prev_index = self.log.len() - 1;
        let prev_term = self.log[prev_index].term;
        let success = append_entries(&mut self.log, prev_index, prev_term, entries);
        if success {
            self.match_index.as_mut().unwrap()[dest] = self.log.len() - 1;
            self.next_index.as_mut().unwrap()[dest] = self.log.len();
        }
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
            let next_idx = (self.next_index.as_ref().unwrap())[follower];
            let prev_index = next_idx - 1;
            let prev_term = if prev_index == 0 {
                0
            } else {
                self.log[prev_index].term
            };
            let entries = self.log[next_idx..].to_vec();
            msgs.push(RaftMessage::AppendEntriesRequest {
                src: dest,
                dest: follower,
                term: self.current_term,
                prev_index,
                prev_term,
                commit_index: self.commit_index,
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
        commit_index: usize,
        entries: Vec<LogEntry<T>>,
    ) -> Vec<RaftMessage<T>> {
        let mut msgs = vec![];
        if term > self.current_term {
            return msgs;
        }
        // Reject request
        if term < self.current_term {
            msgs.push(RaftMessage::AppendEntriesResponse {
                src: dest,
                dest: src,
                term: self.current_term,
                success: false,
                match_index: 0,
            });
            return msgs;
        }
        // Return to follower state
        if term == self.current_term && self.state == ServerStates::Candidate {
            self.state = ServerStates::Follower;
            return msgs;
        }
        let elen = entries.len();
        if commit_index > self.commit_index {
            self.commit_index = commit_index;
            if self.commit_index > self.last_applied {
                // To-do: send AppliedEntries message
                self.last_applied = self.commit_index;
            }
        }
        let success = append_entries(&mut self.log, prev_index, prev_term, entries);
        let match_index = if success {
            prev_index + elen
        } else {
            self.log.len() - 1
        };
        msgs.push(RaftMessage::AppendEntriesResponse {
            src: dest,
            dest: src,
            term: self.current_term,
            success,
            match_index,
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
        let next_index_mut = self.next_index.as_mut().unwrap();
        let match_index_mut = self.match_index.as_mut().unwrap();
        if !success {
            next_index_mut[src] = next_index_mut[src] - 1;
            let mut responses = self.handle_append_entries(dest, vec![src]);
            msgs.append(&mut responses);
        } else {
            next_index_mut[src] = match_index + 1;
            if match_index > match_index_mut[src] {
                match_index_mut[src] = match_index;
            }

            self.advance_commit_index(dest);
        }

        msgs
    }

    fn handle_time_out(&mut self, dest: usize, followers: Vec<usize>) -> Vec<RaftMessage<T>> {
        if self.state != ServerStates::Follower || self.state != ServerStates::Candidate {
            return vec![];
        }
        self.current_term = self.current_term + 1;
        self.voted_for = dest;
        self.votes_responded = Some(vec![dest].iter().cloned().collect());
        self.votes_granted = Some(vec![dest].iter().cloned().collect());
        self.followers = Some(followers.clone());
        self.request_vote(dest, followers)
    }

    fn request_vote(&mut self, dest: usize, followers: Vec<usize>) -> Vec<RaftMessage<T>> {
        let mut msgs = vec![];
        if self.state != ServerStates::Candidate {
            return msgs;
        }
        for follower in followers {
            if self.votes_responded.as_ref().unwrap().contains(&follower) {
                continue;
            }
            let last_log_index = self.log.len() - 1;
            let last_log_term = if last_log_index == 0 {
                0
            } else {
                self.log[last_log_index].term
            };
            msgs.push(RaftMessage::RequestVoteRequest {
                src: dest,
                dest: follower,
                term: self.current_term,
                last_log_index: last_log_index,
                last_log_term: last_log_term,
            });
        }
        msgs
    }

    fn handle_request_vote_request(
        &mut self,
        src: usize,
        dest: usize,
        term: usize,
        last_log_index: usize,
        last_log_term: usize,
    ) -> Vec<RaftMessage<T>> {
        let mut msgs = vec![];
        let last_term = if self.log.len() <= 1 {
            0
        } else {
            self.log.last().unwrap().term
        };
        let log_ok = (last_log_term > last_term)
            || (last_log_term == last_term && last_log_index > self.log.len() - 1);
        let grant =
            (term == self.current_term) && log_ok && (self.voted_for == 0 || self.voted_for == src);
        if term <= self.current_term {
            if grant {
                self.voted_for = src;
            }
            msgs.push(RaftMessage::RequestVoteResponse {
                src: dest,
                dest: src,
                term: self.current_term,
                vote_granted: grant,
            });
        }
        msgs
    }

    fn handle_request_vote_response(
        &mut self,
        src: usize,
        dest: usize,
        term: usize,
        vote_granted: bool,
    ) -> Vec<RaftMessage<T>> {
        if term != self.current_term || self.state != ServerStates::Candidate {
            return vec![];
        }
        self.votes_responded.as_mut().unwrap().insert(src);
        if vote_granted {
            self.votes_granted.as_mut().unwrap().insert(src);
        }
        let quorum = self.followers.as_ref().unwrap().len() + 2 / 2;
        let followers = self.followers.as_ref().unwrap().clone();
        if self.votes_granted.as_ref().unwrap().len() >= quorum {
            self.handle_become_leader(dest, followers);
        }
        vec![]
    }

    fn update_term(&mut self, mterm: usize) {
        if mterm > self.current_term {
            self.current_term = mterm;
            self.state = ServerStates::Follower;
            self.voted_for = 0;
        }
    }

    fn advance_commit_index(&mut self, dest: usize) {
        let mut match_index_cp = self.match_index.as_mut().unwrap().clone();

        match_index_cp.sort_unstable();
        let mid = match_index_cp.len() / 2 as usize;
        let max_agree_index = match_index_cp[mid];
        if self.log[max_agree_index].term >= self.current_term {
            self.commit_index = max_agree_index;
        }
        if self.commit_index > self.last_applied {
            // To-do: send ApplyEntries message
            self.last_applied = self.commit_index;
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
                | RaftMessage::AppendEntriesResponse { dest, .. }
                | RaftMessage::RequestVoteRequest { dest, .. }
                | RaftMessage::RequestVoteResponse { dest, .. }
                | RaftMessage::TimeOut { dest, .. } => dest,
            };
            let server = &mut servers[dest as usize];
            let responses = server.handle_message(msg);
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

        // Check all the logs are identical
        assert!(servers.iter().skip(1).all(|x| { x.log == servers[1].log }));

        // After successful replication, the leader should have commited all its entries
        assert_eq!(servers[1].commit_index, servers[1].log.len() - 1);
    }

    #[test]
    fn test_figure_7() {
        let mut servers = vec![
            RaftServer::new(vec![LogEntry::default()]),
            RaftServer::new(make_log(vec![1, 1, 1, 4, 4, 5, 5, 6, 6, 6])),
            RaftServer::new(make_log(vec![1, 1, 1, 4, 4, 5, 5, 6, 6])),
            RaftServer::new(make_log(vec![1, 1, 1, 4])),
            RaftServer::new(make_log(vec![1, 1, 1, 4, 4, 5, 5, 6, 6, 6, 6])),
            RaftServer::new(make_log(vec![1, 1, 1, 4, 4, 5, 5, 6, 6, 6, 7, 7])),
            RaftServer::new(make_log(vec![1, 1, 1, 4, 4, 4, 4])),
            RaftServer::new(make_log(vec![1, 1, 1, 2, 2, 2, 3, 3, 3, 3, 3])),
        ];

        for server in &mut servers {
            server.current_term = 8;
        }
        servers[1].commit_index = 10;
        run_message(
            RaftMessage::BecomeLeader {
                dest: 1,
                followers: (2..8).collect(),
            },
            &mut servers,
        );

        run_message(
            RaftMessage::ClientRequest {
                dest: 1,
                value: "x".to_string(),
            },
            &mut servers,
        );

        // The first AppendEntries will update leader commit_index
        run_message(
            RaftMessage::AppendEntries {
                dest: 1,
                followers: (2..8).collect(),
            },
            &mut servers,
        );

        // The second AppendEntries will update all followers commit_index
        run_message(
            RaftMessage::AppendEntries {
                dest: 1,
                followers: (2..8).collect(),
            },
            &mut servers,
        );

        assert!(servers.iter().skip(1).all(|x| { servers[1].log == x.log }));
        assert_eq!(servers[1].commit_index, servers[1].log.len() - 1);
        // dbg!(servers[1].match_index.clone());
        // dbg!(servers[1].next_index.clone());
        // for server in servers.iter().skip(1) {
        //     dbg!(server.commit_index);
        // }
        // for server in servers.iter().skip(1) {
        //     dbg!(server.last_applied);
        // }
    }

    #[test]
    fn test_commit() {
        let mut servers = vec![
            RaftServer::new(vec![LogEntry::default()]),
            RaftServer::new(make_log(vec![1, 1, 1, 2, 2])),
            RaftServer::new(make_log(vec![1, 1, 1, 2, 2])),
            RaftServer::new(make_log(vec![1, 1, 1, 2, 2])),
        ];

        for server in &mut servers {
            server.current_term = 2;
        }

        run_message(
            RaftMessage::BecomeLeader {
                dest: 1,
                followers: vec![2, 3],
            },
            &mut servers,
        );

        run_message(
            RaftMessage::ClientRequest {
                dest: 1,
                value: "x".to_string(),
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

        // The leader should have committed the entry. The followers should not because
        // they won't learn about the commit index until leader send them another AppendEntries
        assert_eq!(servers[1].commit_index, 6);
        assert_eq!(servers[1].last_applied, 6);
        assert!(servers.iter().skip(2).all(|x| { x.commit_index == 5 }));
        assert!(servers.iter().skip(2).all(|x| { x.last_applied == 5 }));

        // The followers will commit and apply after leader send another AppendEntries
        run_message(
            RaftMessage::AppendEntries {
                dest: 1,
                followers: vec![2, 3],
            },
            &mut servers,
        );
        assert!(servers.iter().skip(2).all(|x| { x.commit_index == 6 }));
        assert!(servers.iter().skip(2).all(|x| { x.last_applied == 6 }));
    }

    #[test]
    fn test_figure_6_election() {
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

        // Test: let server 1 time out to become a candidate. It should win the election with all votes
        run_message(
            RaftMessage::TimeOut {
                dest: 1,
                followers: (2..6).collect(),
            },
            &mut servers,
        );
        assert_eq!(servers[1].state, ServerStates::Leader);
    }
}
