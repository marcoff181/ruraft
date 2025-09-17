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

use core::fmt::Debug;

pub use crate::raftmodel::*;

use verus_builtin_macros::*;
use verus_state_machines_macros::tokenized_state_machine;
use vstd::prelude::*;
use vstd::prelude::nat;

use std::collections::{VecDeque, HashSet};

verus! {

#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct Dummy {}

fn verification_harness() {
    let nservers = 5;

    let tracked(
        num_of_server,
        messages,
        servers,
        elections,
        allLogs,
        current_term,
        state,
        voted_for,
        log,
        commit_index,
        votes_responded,
        votes_granted,
        voter_log,
        next_index,
        match_index
    ) = RAFT::Instance::<Dummy>::initialize(set![1,2,3,4,5]);


    let log = create_empty_log::<Dummy>();
    let mut raft_servers = Vec::new(); 
    for x in iter: 1..(nservers+1) {
        raft_servers.push(RaftServer::new(log.clone()));
    };
    //  let mut messages = VecDeque::new();
    //  messages.push_back(RaftMessage::TimeOut {
    //         dest: 1,
    //         followers: (2..6).collect(),
    //     });
    //  while let Some(msg) = messages.pop_front() {
    //      let dest = match msg {
    //          RaftMessage::ClientRequest { dest, .. }
    //          | RaftMessage::BecomeLeader { dest, .. }
    //          | RaftMessage::AppendEntries { dest, .. }
    //          | RaftMessage::AppendEntriesRequest { dest, .. }
    //          | RaftMessage::AppendEntriesResponse { dest, .. }
    //          | RaftMessage::RequestVoteRequest { dest, .. }
    //          | RaftMessage::RequestVoteResponse { dest, .. }
    //          | RaftMessage::TimeOut { dest, .. } => dest,
    //      };
    //      let server = &mut servers[dest as usize];
    //      let responses = server.handle_message(msg);
    //      messages.append(&mut responses.into_iter().collect());
    //  }
    //
    // assert_eq!(*servers[1].server_state(), ServerState::Leader);

    // let mut fuel = 5;
    // while fuel > 0 {
    //     invariant(tracked_token.borrow().election_safety());
    //     fuel -= 1;
    //
    // }
}

#[verifier::external_trait_specification]
pub trait ExDefault where Self: core::marker::Sized {
    type ExternalTraitSpecificationFor: Default;

    fn default() -> Self where Self: core::marker::Sized;
}

pub struct ElectionRecord<T: Sized + Clone + PartialEq + Eq + Debug + Default> {
    pub term : nat,
    pub leader: nat,
    pub log: Seq<LogEntry<T>>,
    pub votes: Set<nat>,
    pub voter_log:Map<nat,Seq<LogEntry<T>>>,
}


tokenized_state_machine!{
    #[verifier::reject_recursive_types(T)]
    RAFT<T: Sized + Clone + PartialEq + Eq + Debug + Default>{
        fields{
            #[sharding(constant)]
            pub num_of_server : nat,

            // global variables
            #[sharding(set)]
            pub messages: Set<RaftMessage<T>>,

            #[sharding(variable)]
            pub servers: Set<nat>,

            #[sharding(set)]
            pub elections:Set<ElectionRecord<T>>,

            #[sharding(map)]
            pub allLogs: Map<nat,LogEntry<T>>,

            // per-server variables
            #[sharding(map)]
            pub current_term: Map<nat,nat>,

            #[sharding(map)]
            pub state: Map<nat,ServerState>,

            #[sharding(map)]
            pub voted_for: Map<nat,Option<nat>>,

            #[sharding(map)]
            pub log: Map<nat,Seq<LogEntry<T>>>,

            // \* The index of the latest entry in the log the state machine may apply.
            // VARIABLE commitIndex
            #[sharding(map)]
            pub commit_index: Map<nat,nat>,

            // candidates only variables
            
            //The set of servers from which the candidate has received a RequestVote response in its currentTerm.
            #[sharding(map)]
            pub votes_responded: Map<nat,Set<nat>>,

            //The set of servers from which the candidate has received a vote in its currentTerm.
            #[sharding(map)]
            pub votes_granted: Map<nat,Set<nat>>,

            // A history variable used in the proof. This would not be present in an implementation.Function from each server that voted for this candidate in its currentTerm to that voter's log.
            #[sharding(map)]
            pub voter_log: Map<nat,Map<nat,Seq<LogEntry<T>>>>,

            // the next entry to send to each follower
            #[sharding(map)]
            pub next_index: Map<nat,Map<nat,nat>>,

            // \* The latest entry that each follower has acknowledged is the same as the
            // \* leader's. This is used to calculate commitIndex on the leader.
            #[sharding(map)]
            pub match_index: Map<nat,Map<nat,nat>>,
        }

        // there can never be two leaders at the same time
        // #[invariant]
        // pub fn election_safety(&self) -> bool { 
        //     forall |i: nat, j: nat| 
        //             i != j && self.state.dom().contains(i) && self.state.dom().contains(j)
        //             ==> !(#[trigger] self.state.get(i) == Some(ServerState::Leader) && #[trigger] self.state.get(j) == Some(ServerState::Leader))
        // }


        #[inductive(initialize)]
        fn initialize_inductive(post: Self, servers: Set<nat>) 
        {
        }

        init!{
            initialize(servers: Set<nat>)
            {
                let num_nodes = servers.len();
                require(num_nodes > 1);

                init num_of_server = num_nodes;
                init servers = servers;
                init messages = Set::empty();
                init elections = Set::empty();
                init allLogs = Map::empty();
                init current_term = Map::new(|i:nat| servers has i , |i:nat| 1);
                init state = Map::new(|i:nat| servers has i, |i:nat| ServerState::Follower);
                init voted_for = Map::new(|i:nat| servers has i, |i:nat| None);
                init log = Map::new(|i:nat| servers has i, |i:nat| Seq::empty());
                init commit_index = Map::new(|i:nat| servers has i, |i:nat| 0);
                init votes_responded = Map::empty();
                init votes_granted = Map::new(|i:nat| servers has i, |i:nat| Set::empty());
                init voter_log = Map::empty();
                init next_index = Map::new(|i:nat| servers has i, |i:nat| Map::new(|j:nat| servers has j, |j:nat| 1));
                init match_index = Map::new(|i:nat| servers has i, |i:nat| Map::new(|j:nat| servers has j, |j:nat| 0));
            }
        }



        #[inductive(handle_request_vote_request)]
        fn handle_request_vote_request_inductive(pre: Self, post: Self, m: RaftMessage<T>) { }

        transition!{
            handle_request_vote_request(m:RaftMessage<T>){
                remove messages -= set { m };
                require let  RaftMessage::RequestVoteRequest { src, dest, term, last_log_index, last_log_term } = m;
                // remove messages -= [ msg_id => let
                // RaftMessage::RequestVoteRequest {
                //     src,
                //     dest,
                //     term,
                //     last_log_index,
                //     last_log_term,
                // }];

                have log >= [dest as nat =>  let current_log  ];
                let my_last_log_index = current_log.last();
                have current_term >= [dest as nat =>  let my_current_term  ];
                let my_last_log_term = my_last_log_index.term;
                remove voted_for -= [dest as nat =>  let i_voted_for  ];

                // LET logOk == \/ m.mlastLogTerm > LastTerm(log[i])
                //              \/ /\ m.mlastLogTerm = LastTerm(log[i])
                //                 /\ m.mlastLogIndex >= Len(log[i])
                let log_ok = last_log_term > my_last_log_term 
                        || (last_log_term == my_last_log_term && last_log_index >= current_log.len() );   

                //     grant == /\ m.mterm = currentTerm[i]
                //              /\ logOk
                //              /\ votedFor[i] \in {Nil, j}
                let grant = term == my_current_term
                        &&  log_ok
                        && (i_voted_for == None::<nat> || i_voted_for == Some(src as nat)); 

                // TODO: confirm this makes sense
                // IN /\ m.mterm <= currentTerm[i]
                require(term <= my_current_term);

                //    /\ \/ grant  /\ votedFor' = [votedFor EXCEPT ![i] = j]
                //       \/ ~grant /\ UNCHANGED votedFor
                let updated_voted_for = 
                match grant{
                    true => Some(src as nat),
                    false => i_voted_for
                };
                add voted_for += [dest as nat => updated_voted_for]; 

                //    /\ Reply([mtype        |-> RequestVoteResponse,
                //              mterm        |-> currentTerm[i],
                //              mvoteGranted |-> grant,
                //              \* mlog is used just for the `elections' history variable for
                //              \* the proof. It would not exist in a real implementation.
                //              mlog         |-> log[i],
                //              msource      |-> i,
                //              mdest        |-> j],
                //              m)
                let response =RaftMessage::RequestVoteResponse{
                    src : dest,
                    dest:src,
                    term: my_current_term as usize,
                    vote_granted : grant,
                };
                
                // have to do with sets at the moment because it's apparently very hard to generate
                // fresh ids
                remove messages -= set{response};
                add messages += set{response};

                //    /\ UNCHANGED <<state, currentTerm, candidateVars, leaderVars, logVars>>
            }
        }

       
        #[inductive(handle_request_vote_response)]
        fn handle_request_vote_response_inductive(pre: Self, post: Self, m: RaftMessage<T>) { }
       
        transition!{
            handle_request_vote_response(m:RaftMessage<T>){
                remove messages -= set { m };
                require let  RaftMessage::RequestVoteResponse { src, dest, term, vote_granted } = m;

                have current_term >= [dest as nat =>  let my_current_term  ];

                // \* This tallies votes even when the current state is not Candidate, but
                // \* they won't be looked at, so it doesn't matter.
                // /\ m.mterm = currentTerm[i]
                require(term == my_current_term);

                // /\ votesResponded' = [votesResponded EXCEPT ![i] =
                //                           votesResponded[i] \cup {j}]
                remove votes_responded -= [dest as nat=> let dest_votes_responded];
                add votes_responded += [ dest as nat=> (dest_votes_responded.insert(src as nat))];


                // /\ \/ /\ m.mvoteGranted
                //       /\ votesGranted' = [votesGranted EXCEPT ![i] =
                //                               votesGranted[i] \cup {j}]
                remove votes_granted -= [dest as nat => let mut dest_votes_granted];
                let new_dest_votes_granted =  match vote_granted {
                    true => {dest_votes_granted.insert(src as nat)}
                    false => {dest_votes_granted}
                };
                add votes_granted += [dest as nat => new_dest_votes_granted ];

                //       /\ voterLog' = [voterLog EXCEPT ![i] =
                //                           voterLog[i] @@ (j :> m.mlog)]
                //    \/ /\ ~m.mvoteGranted
                //       /\ UNCHANGED <<votesGranted, voterLog>>
                remove voter_log -= [dest as nat => let mut dest_voter_log];
                let new_dest_voter_log =  match vote_granted {
                    // TODO: need to add message mlog
                    true => {dest_voter_log.insert(src as nat,Seq::empty() )}
                    false => {dest_voter_log}
                };
                add voter_log += [dest as nat => new_dest_voter_log ];

                // /\ Discard(m)
                remove messages -= set{m};
                // /\ UNCHANGED <<serverVars, votedFor, leaderVars, logVars>>
            }
        }

        // ----------- functions that represent internal behavior not related to message receiving logic

        #[inductive(become_leader)]
        fn become_leader_inductive(pre: Self, post: Self, src:nat) {
        }

        transition!{
            become_leader(src: nat){
                have log >= [ src as nat => let src_log]; 
                have current_term >= [ src as nat => let src_current_term];
                have voter_log >= [src as nat => let src_voter_log];

                // /\ state[i] = Candidate
                remove state -= [src as nat => ServerState::Candidate];

                // /\ votesGranted[i] \in Quorum
                have votes_granted >= [src as nat => let src_votes_granted];
                let threshold = pre.servers.len() / 2;  
                let quorum = Set::new(|s: Set<nat>|   
                    s.subset_of(pre.servers) && s.finite() && s.len() > threshold  
                );  
                require (quorum has src_votes_granted);

                // /\ state'      = [state EXCEPT ![i] = Leader]
                add state += [src as nat => ServerState::Leader];

                // /\ nextIndex'  = [nextIndex EXCEPT ![i] =
                //                      [j \in Server |-> Len(log[i]) + 1]]
                remove next_index -= [src as nat => let _];
                let src_next_index = Map::new(|j:nat| pre.servers has j ,|j:nat| src_log.len()) ;
                add next_index += [src as nat => src_next_index];

                // /\ matchIndex' = [matchIndex EXCEPT ![i] =
                //                      [j \in Server |-> 0]]
                remove match_index -= [src as nat => let _];
                let src_match_index = Map::new(|j:nat| pre.servers has j ,|j:nat| 0nat) ;
                add match_index += [src as nat => src_match_index];

                // /\ elections'  = elections \cup
                //                      {[eterm     |-> currentTerm[i],
                //                        eleader   |-> i,
                //                        elog      |-> log[i],
                //                        evotes    |-> votesGranted[i],
                //                        evoterLog |-> voterLog[i]]}
                let new_election_record = ElectionRecord{
                    term : src_current_term,
                    leader: src as nat,
                    log: src_log,
                    votes: src_votes_granted,
                    voter_log:src_voter_log,
                };

                // TODO: see if this can be removed
                remove elections -= set { new_election_record};
                add elections += set { new_election_record};

                // /\ UNCHANGED <<messages, currentTerm, votedFor, candidateVars, logVars>>
            }
        }

        #[inductive(request_vote)]
        fn request_vote_inductive(pre: Self, post: Self, src:nat,dest:nat) {

        }

        transition!{
            request_vote(src:nat,dest:nat){
                // /\ state[i] = Candidate
                have state >= [src as nat => ServerState::Candidate]; 
                // /\ j \notin votesResponded[i]
                have votes_responded >= [src as nat => let voted_for_me];
                require(! voted_for_me has dest);
                // /\ Send([mtype         |-> RequestVoteRequest,
                //          mterm         |-> currentTerm[i],
                //          mlastLogTerm  |-> LastTerm(log[i]),
                //          mlastLogIndex |-> Len(log[i]),
                //          msource       |-> i,
                //          mdest         |-> j])
                have current_term >= [src as nat => let term];
                have log >= [dest as nat =>  let current_log  ];
                let last_log_index = current_log.last();
                let last_log_term =last_log_index.term;
                let msg =RaftMessage::RequestVoteRequest{
                    // TODO: confirm that last_log_index is correctly defined here
                     src:src as usize, dest:dest as usize, term : term as usize, last_log_index:current_log.len() as usize, last_log_term 
                };

                // have to do with sets at the moment because it's apparently very hard to generate
                // fresh ids
                remove messages -= set{msg};
                add messages += set{msg};

                // /\ UNCHANGED <<serverVars, candidateVars, leaderVars, logVars>>
            }
        }

        // transition!{
        //     handle_client_request(m:RaftMessage<T>){
        //         // remove messages -= [ msg_id => let  RaftMessage::ClientRequest{dest,value}];
        //         remove messages -= set { m };
        //         require let  RaftMessage::ClientRequest{dest,value} = m;
        //
        //         have current_term >= [dest as nat => let term];
        //
        //         // /\ state[i] = Leader
        //         have state >= [dest as nat => ServerState::Leader];
        //
        //         // /\ LET entry == [term  |-> currentTerm[i],
        //         //                  value |-> v]
        //         //        newLog == Append(log[i], entry)
        //         //    IN  log' = [log EXCEPT ![i] = newLog]
        //         remove log -= [dest as nat =>  let current_log  ];
        //         add log += [dest as nat => { current_log.push(LogEntry::<T>{term : term as usize ,item:value}) }];
        //
        //         // /\ UNCHANGED <<messages, serverVars, candidateVars,
        //         //                leaderVars, commitIndex>>
        //     }
        // }
        //
    }
}

} // verus!
