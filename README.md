# River Raft
A test implementation of raft p2p

## Raft
Nodes can be in one of three states: Follower, Candidate and Leader.

All Nodes start in follower state.
When a node doesnt hear from a leader for some fixed time quanta it turns into a candidate.

The Candidate then requests votes from other follower nodes.
Once they vote, if the votes total to a majority then the node becomes a leader.

Hence its called **Leader Election**

Any change to the data must go through a leader now.
The Leader maintains a log of the changes but when you make a change it is **uncommited** in the nodes db.
The Node will then **replicate** the change to follower nodes.
The Node waits until the majority of followers respond with confirmation that they have replicated the entry.
Only then it commits the change and then sends a response to followers saying the change has been commited.
This is how the system comes to consensus and the process is called **Log Replication**

### Elections
There are two timeout settings that control election.

1. Election Timeout
The time a follower has to wait before becoming a candidate.
It's randomized between 150-300ms.

On becoming a candidate it starts a new election term and votes for itself.
it also sends vote request messages to follower nodes, the followers vote for the candidate
if they havent already voted in the term and the node resets it's election timeout.

Once it has majority votes it becomes the leader.

The leader starts sending Append Entries messages to the followers.
These messages are sent in intervals specified by the heartbeat timeout.

If you're wondering what a heartbeat time out is it's basically to know if the leader is still active, 
its a fixed time period of 150-300ms where followers expect a heartbeat from the leader, 
if not received the follower tries to become a candidate and a new term is started.

The followers then respond with Append Entries messages.

The leaders term continues until followers stops receiving heartbeat from it and start a new term.

Taking majority votes ensures only one leader is elected per term.

If two candidates get elected at the same time then a split vote can occur.

In this case again the majority voted leader is chosen but if the votes are equal then a re-election is conducted, 
this continues until a leader is elected by majority vote.

### Log Replication
Once leader is elected it has to replicate changes to all folowers. We use Append Entries message to do so.

The leader is periodically heartbeating as it should.

User X sends a change to leader which is registered in it's local changelog.

Leader replicates the change in the next heartbeat to its followers.

The entry is commited only once majority of followers reply with confirmation that they have replicated the change
and a response is sent to followers and User X.

Even if the network gets partioned it can stay consistent guaranteeing strong consistency.

If there are 5 nodes(A,B,C,D,E), assume A is leader and A and B get partioned. C, D, E will start a new election term 
once their timeout resets causing two leaders to exist in two different terms. Essentially a fork.

If someone tries to update Node A it will fail and won't be commited because there won't be a majority vote however with Node C as the leader in 
the other parition if you update it's value then it can get a majority vote.

Now let's imagine the partition is fixed. In this case Node A will see that the term of Node C is higher and step down from the leader role.
Both Node A and B will rollback uncommited changes and replicate the leaders logs.

We now have a consistent log.
