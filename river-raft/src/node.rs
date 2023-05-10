use parking_lot::RwLock;
use rand::Rng;
use std::{net::SocketAddr, sync::Arc, thread::Builder};

use crate::logstore::{LogStore, Protocol};

pub const ELECTION_TIMEOUT_MAX: u64 = 300;
pub const ELECTION_TIMEOUT_MIN: u64 = 150;
pub const PACKET_SIZE: usize = 1500;

#[derive(Clone)]
pub struct Node {
    pub id: u64,
    pub socket: Arc<SocketAddr>,
    pub state: Arc<RwLock<NodeState>>,
    pub term: u64,
    pub current_leader: Arc<RwLock<SocketAddr>>,
    pub timeout: u64,
    pub log_store: Arc<RwLock<LogStore>>,
}
impl Node {
    pub fn new(id: u64, socket: SocketAddr, term: u64, log_store: Arc<RwLock<LogStore>>) -> Self {
        Node {
            id,
            socket: Arc::new(socket),
            state: Arc::new(RwLock::new(NodeState::Follower)),
            term,
            current_leader: Arc::new(RwLock::new(socket)),
            timeout: get_election_timeout(),
            log_store,
        }
    }
    pub fn start(self) {
        let mut threads = vec![];
        let (sender_comms, receiver_comms) = crossbeam::channel::unbounded::<[u8; PACKET_SIZE]>();
        let (sender_process, receiver_process) = crossbeam::channel::unbounded();
        // let should_elect_new = Arc::new(RwLock::new(false));
        let node_state = self.state.clone();
        let node_state2 = self.state.clone();
        let node_state3 = self.state.clone();
        let timeout = self.timeout.clone();
        let current_leader1 = self.current_leader.clone();
        // let current_leader2 = self.current_leader.clone();
        let socket1 = self.socket.clone();
        let socket2 = self.socket.clone();
        let node_ticker = Arc::new(RwLock::new(std::time::Instant::now()));
        let ticker1 = node_ticker.clone();
        // let ticker2 = node_ticker.clone();
        threads.push(
            Builder::new()
                .name("election ticker".into())
                .spawn(move || loop {
                    if (timeout as i64) - node_ticker.read().elapsed().as_millis() as i64 <= 0 {
                        match *node_state2.read() {
                            NodeState::Follower => {
                                *node_state2.write() = NodeState::Candidate;
                            }
                            NodeState::Candidate => {}
                            NodeState::Leader => {}
                        }

                        *node_ticker.write() = std::time::Instant::now();
                    }
                })
                .unwrap(),
        );
        threads.push(
            Builder::new()
                .name("receiver".into())
                .spawn(move || loop {
                    let mut buf = [0u8; PACKET_SIZE];
                    if let Ok(_data) = std::net::UdpSocket::bind(*socket1)
                        .unwrap()
                        .recv_from(&mut buf)
                    {
                        let message = bincode::deserialize::<Protocol>(&buf).unwrap();
                        sender_process.send(message).unwrap();
                    }
                })
                .unwrap(),
        );
        threads.push(
            Builder::new()
                .name("processor".into())
                .spawn(move || loop {
                    if let Ok(data) = receiver_process.try_recv() {
                        match data {
                            Protocol::AppendEntryRequest(_data) => {
                                *ticker1.write() = std::time::Instant::now();
                            }
                            Protocol::AppendEntryResponse(_data) => {
                                *ticker1.write() = std::time::Instant::now();
                            }
                            Protocol::RequestVote(data) => match *self.state.read() {
                                NodeState::Follower => {
                                    if self.should_vote(data.term) {
                                        sender_comms
                                            .send(Protocol::vote_response(self.term, true))
                                            .unwrap();
                                    } else {
                                        sender_comms
                                            .send(Protocol::vote_response(self.term, false))
                                            .unwrap();
                                    }
                                }
                                _ => {}
                            },
                            Protocol::RespondVote(data) => {
                                *ticker1.write() = std::time::Instant::now();
                            }
                            Protocol::Ping(_data) => {
                                *ticker1.write() = std::time::Instant::now();
                                // sender_comms.send(Protocol::Pong(self.id)).unwrap();
                            }
                        }
                    }
                })
                .unwrap(),
        );
        threads.push(
            Builder::new()
                .name("sender".into())
                .spawn(move || loop {
                    if let Ok(data) = receiver_comms.try_recv() {
                        std::net::UdpSocket::bind(*socket2)
                            .unwrap()
                            .send_to(&data, *current_leader1.read())
                            .unwrap();
                    }
                })
                .unwrap(),
        );

        for thread in threads {
            thread.join().unwrap();
        }
    }
    pub fn should_vote(&self, term: u64) -> bool {
        if term > self.term {
            true
        } else {
            false
        }
    }
}
#[derive(Debug, Clone)]
pub enum NodeState {
    Follower,
    Candidate,
    Leader,
}
pub fn get_election_timeout() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(ELECTION_TIMEOUT_MIN..ELECTION_TIMEOUT_MAX)
}
