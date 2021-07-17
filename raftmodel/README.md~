`raftmodel` aims to provide the rust implementations of logic model for the raft consensus algorithm.

# The big picture
`raftmodel` is a crate to provide a pure logic model for the raft algorithm.
It aims to strictly follow raft formal TLA+ specification ([raft.tla](https://github.com/ongardie/raft.tla/blob/master/raft.tla)).

To use `raftmodel`, user interacts with raft by calling `handle_message` method in `RaftServer`. The input of `handle_message`
is `RaftMessage` and the output is `Vec<RaftMessage>`.
