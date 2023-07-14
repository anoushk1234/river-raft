use crossbeam::channel::Sender;
use parking_lot::RwLock;
use rand::Rng;
use std::{net::SocketAddr, sync::Arc, thread::Builder};

use crate::logstore::{Data, LogStore, NodeInstance, Protocol, RequestVote};

pub const ELECTION_TIMEOUT_MAX: u64 = 300;
pub const ELECTION_TIMEOUT_MIN: u64 = 150;
pub const PACKET_SIZE: usize = 1500;

#[derive(Clone)]
pub struct Node {
    pub id: u64,
    pub sender_socket: Arc<SocketAddr>,
    pub receiver_socket: Arc<SocketAddr>,
    pub state: Arc<RwLock<NodeState>>,
    pub term: u64,
    pub current_leader: Arc<RwLock<SocketAddr>>,
    pub timeout: u64,
    pub log_store: Arc<RwLock<LogStore>>,
    pub vote: u64, // pub election_term: Option<ElectionTerm>,
}
impl Node {
    pub fn new(
        id: u64,
        sender_socket: SocketAddr,
        receiver_socket: SocketAddr,
        term: u64,
        log_store: Arc<RwLock<LogStore>>,
    ) -> Self {
        Node {
            id,
            sender_socket: Arc::new(sender_socket),
            receiver_socket: Arc::new(receiver_socket),
            state: Arc::new(RwLock::new(NodeState::Follower)),
            term,
            current_leader: Arc::new(RwLock::new(sender_socket)),
            timeout: get_election_timeout(),
            log_store,
            vote: 0, // zero means none
        }
    }
    pub fn init(
        id: u64,
        sender_socket: &str,
        receiver_socket: &str,
        log_store: Arc<RwLock<LogStore>>,
    ) -> Self {
        Node {
            id,
            sender_socket: Arc::new(sender_socket.parse().unwrap()),
            receiver_socket: Arc::new(receiver_socket.parse().unwrap()),
            state: Arc::new(RwLock::new(NodeState::Follower)),
            term: 0,
            current_leader: Arc::new(RwLock::new(sender_socket.parse().unwrap())),
            timeout: get_election_timeout(),
            log_store,
            vote: 0,
        }
    }
    pub fn start(&mut self) {
        let mut threads = vec![];
        let (sender_comms, receiver_comms) = crossbeam::channel::unbounded::<[u8; PACKET_SIZE]>();
        let (sender_process, receiver_process) = crossbeam::channel::unbounded();
        // let should_elect_new = Arc::new(RwLock::new(false));
        // let node_state = self.state.clone();

        let sender_comms2 = sender_comms.clone();
        let node_id = self.id;
        let node_state2 = self.state.clone();
        let node_state3 = self.state.clone();
        let timeout = self.timeout.clone();
        let current_leader1 = self.current_leader.clone();
        // let current_leader2 = self.current_leader.clone();
        // let socket1 = self.receiver_socket.clone();
        // let socket2 = self.socket.clone();
        let node_ticker = Arc::new(RwLock::new(std::time::Instant::now()));
        let ticker1 = node_ticker.clone();
        // let ticker2 = node_ticker.clone();
        let mut node = self.clone();
        let node2 = self.clone();
        let node3 = self.clone();
        threads.push(
            Builder::new()
                .name("election ticker".into())
                .spawn(move || loop {
                    if (timeout as i64) - node_ticker.read().elapsed().as_millis() as i64 <= 0 {
                        println!("node {:?} state {:?}", node_id, node_state2);
                        let read_st = node_state2.read();
                        match *read_st {
                            NodeState::Follower => {
                                drop(read_st);
                                let mut write_st = node_state2.write();
                                *write_st = NodeState::Candidate;
                                drop(write_st);
                            }
                            NodeState::Candidate => {
                                node.term += 1;
                                node3.request_vote(sender_comms2.clone(), node.term, node.id);
                            }
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
                    if let Ok(_data) = std::net::UdpSocket::bind(*node2.receiver_socket)
                        .unwrap()
                        .recv_from(&mut buf)
                    {
                        let message = bincode::deserialize::<Protocol>(&buf).unwrap();
                        // println!("recvd msg {:?}", message);
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
                            Protocol::RequestVote(data) => match *node_state3.read() {
                                NodeState::Follower => {
                                    *ticker1.write() = std::time::Instant::now();
                                    println!("RequestVote {:?}", data);
                                    if node.should_vote(data.term) {
                                        sender_comms
                                            .send(Protocol::vote_response(node.term, true))
                                            .unwrap();
                                    } else {
                                        sender_comms
                                            .send(Protocol::vote_response(node.term, false))
                                            .unwrap();
                                    }
                                }
                                NodeState::Candidate => {}
                                _ => {}
                            },
                            Protocol::RespondVote(data) => {
                                *ticker1.write() = std::time::Instant::now();
                                println!("RespondVote {:?}", data);
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
                        let read_store = node2.log_store.read();
                        let peers = read_store.get_peers();
                        for peer in peers {
                            if peer.id != node2.id {
                                println!("node2.socket {:?}", node2.sender_socket);
                                std::net::UdpSocket::bind(*node2.sender_socket)
                                    .unwrap()
                                    .send_to(&data, peer.receiver_ip)
                                    .expect("failed to send in sender");
                            }
                        }
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
    pub fn request_vote(&self, sender: Sender<[u8; PACKET_SIZE]>, term: u64, id: u64) {
        let req = Protocol::vote_request(term, id);
        sender.send(req);
    }
    // pub fn vote(&self, term: u64, id:u64, sender: Sender<Protocol>) -> bool{
    //     if self.should_vote(term){
    //         let message = Protocol::RequestVote(())
    //         sender.send()
    //     }
    // }
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
