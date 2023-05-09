use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
    thread::Builder,
};

use rand::Rng;

use crate::logstore::{LogStore, Protocol};
const ELECTION_TIMEOUT_MAX: u64 = 300;
const ELECTION_TIMEOUT_MIN: u64 = 150;
const PACKET_SIZE: usize = 3000;
pub struct Node {
    pub id: u64,
    pub socket: SocketAddr,
    pub state: Arc<RwLock<NodeState>>,
    pub term: u32,
    pub current_leader: SocketAddr,
    pub timeout: u64,
    pub log_store: Arc<RwLock<LogStore>>,
}
impl Node {
    pub fn new(id: u64, socket: SocketAddr, term: u32, log_store: Arc<RwLock<LogStore>>) -> Self {
        Node {
            id,
            socket,
            state: Arc::new(RwLock::new(NodeState::Follower)),
            term,
            current_leader: socket,
            timeout: get_election_timeout(),
            log_store,
        }
    }
    pub fn start(self) {
        let mut threads = vec![];
        let (sender_comms, receiver_comms) = crossbeam::channel::unbounded();
        let (sender_process, receiver_process) = crossbeam::channel::unbounded();
        let should_elect_new = Arc::new(RwLock::new(false));
        let node_state = self.state.clone();
        let node_state2 = self.state.clone();
        let node_state3 = self.state.clone();
        let mut node_ticker = std::time::Instant::now();
        threads.push(
            Builder::new()
                .name("receiver".into())
                .spawn(move || loop {
                    match *node_state.read().unwrap() {
                        NodeState::Follower => {
                            let mut buf = [0u8; PACKET_SIZE];
                            if let Ok(_data) = std::net::UdpSocket::bind(self.socket)
                                .unwrap()
                                .recv_from(&mut buf)
                            {
                                let message = bincode::deserialize::<Protocol>(&buf).unwrap();
                                sender_process.send(message).unwrap();
                            }
                        }
                        NodeState::Candidate => {}
                        NodeState::Leader => {}
                    }
                })
                .unwrap(),
        );
        threads.push(
            Builder::new()
                .name("processor".into())
                .spawn(move || loop {
                    match *self.state.read().unwrap() {
                        NodeState::Follower => {
                            if let Ok(data) = receiver_process.try_recv() {
                                match data {
                                    Protocol::AppendEntryRequest(_data) => {}
                                    Protocol::AppendEntryResponse(_data) => {}
                                    Protocol::RequestVote(_data) => {}
                                    Protocol::RespondVote(_data) => {}
                                    Protocol::Ping(_data) => {
                                        node_ticker = std::time::Instant::now();
                                        sender_comms.send(Protocol::Pong(self.id)).unwrap();
                                    }
                                    Protocol::Pong(_data) => {}
                                }
                            }
                        }
                        NodeState::Candidate => {}
                        NodeState::Leader => {}
                    }
                })
                .unwrap(),
        );
        threads.push(
            Builder::new()
                .name("sender".into())
                .spawn(move || loop {
                    match *node_state3.read().unwrap() {
                        NodeState::Follower => {
                            if let Ok(data) = receiver_comms.try_recv() {
                                std::net::UdpSocket::bind(self.socket)
                                    .unwrap()
                                    .send_to(
                                        &bincode::serialize(&data).unwrap(),
                                        self.current_leader,
                                    )
                                    .unwrap();
                            }
                        }
                        NodeState::Candidate => {}
                        NodeState::Leader => {}
                    }
                })
                .unwrap(),
        );
        threads.push(
            Builder::new()
                .name("election".into())
                .spawn(move || loop {
                    match *node_state2.read().unwrap() {
                        NodeState::Follower => {
                            if self.timeout - node_ticker.elapsed().as_millis() as u64 <= 0 {
                                *node_state2.write().unwrap() = NodeState::Candidate;
                            }
                        }
                        NodeState::Candidate => {}
                        NodeState::Leader => {}
                    }
                })
                .unwrap(),
        );

        for thread in threads {
            thread.join().unwrap();
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
