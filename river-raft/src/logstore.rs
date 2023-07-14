use indexmap::{self, IndexMap};

use serde_derive::{Deserialize, Serialize};

use crate::node::PACKET_SIZE;

#[derive(Clone)]
pub struct LogStore {
    /// index -> (term, value to be stored)
    pub entries: IndexMap<u64, Entry>,
}
impl LogStore {
    pub fn create_entrypoint(nodes: Vec<NodeInstance>) -> Self {
        let mut log_store = LogStore {
            entries: IndexMap::new(),
        };
        nodes.into_iter().enumerate().for_each(|(i, node)| {
            if !log_store.entries.contains_key(&(i as u64)) {
                let entry = (Metadata::default(), Data::NodeInstance(node));
                log_store.entries.insert(i as u64, entry);
            } else {
                println!("Node with ID already exists")
            }
        });
        log_store
    }
    pub fn get_peers(&self) -> Vec<&NodeInstance> {
        self.entries
            .values()
            .into_iter()
            .map(|d| match &d.1 {
                Data::NodeInstance(n) => Some(n),
                Data::Message(_) => None,
            })
            .flatten()
            .collect::<Vec<&NodeInstance>>()
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub enum Protocol {
    AppendEntryRequest(AppendEntryRequest),
    AppendEntryResponse(AppendEntryResponse),
    RequestVote(RequestVote),
    RespondVote(RespondVote),
    Ping(u64),
    // Pong(u64),
}
impl Protocol {
    pub fn heartbeat(socket: std::net::SocketAddr, term: u64, leader_id: u64) -> [u8; PACKET_SIZE] {
        let p = Protocol::Ping(leader_id);
        let mut buf = [0u8; PACKET_SIZE];
        let message = bincode::serialize(&p).unwrap();
        buf[..message.len()].copy_from_slice(&message);
        buf
    }
    pub fn vote_request(term: u64, candidate_id: u64) -> [u8; PACKET_SIZE] {
        let p = Protocol::RequestVote(RequestVote { term, candidate_id });
        let mut buf = [0u8; PACKET_SIZE];
        let message = bincode::serialize(&p).unwrap();
        buf[..message.len()].copy_from_slice(&message);
        buf
    }
    pub fn vote_response(term: u64, vote: bool) -> [u8; PACKET_SIZE] {
        let p = Protocol::RespondVote(RespondVote { term, vote });
        let mut buf = [0u8; PACKET_SIZE];
        let message = bincode::serialize(&p).unwrap();
        buf[..message.len()].copy_from_slice(&message);
        buf
    }
    pub fn append_entry_request(term: u64, leader_id: u64, entry: Entry) -> [u8; PACKET_SIZE] {
        let p = Protocol::AppendEntryRequest(AppendEntryRequest {
            term,
            leader_id,
            entry,
        });
        let mut buf = [0u8; PACKET_SIZE];
        let message = bincode::serialize(&p).unwrap();
        buf[..message.len()].copy_from_slice(&message);
        buf
    }
    pub fn append_entry_response(
        socket: std::net::SocketAddr,
        term: u64,
        result: bool,
        conflict: Option<Conflict>,
    ) -> [u8; PACKET_SIZE] {
        let p = Protocol::AppendEntryResponse(AppendEntryResponse {
            term,
            result,
            conflict,
        });
        let mut buf = [0u8; PACKET_SIZE];
        let message = bincode::serialize(&p).unwrap();
        buf[..message.len()].copy_from_slice(&message);
        buf
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppendEntryRequest {
    pub leader_id: u64,
    pub term: u64,
    pub entry: Entry,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct AppendEntryResponse {
    pub term: u64,
    pub result: bool,
    pub conflict: Option<Conflict>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct RequestVote {
    pub term: u64,
    pub candidate_id: u64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct RespondVote {
    pub term: u64,
    pub vote: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeInstance {
    pub sender_ip: std::net::SocketAddr,
    pub receiver_ip: std::net::SocketAddr,
    pub id: u64,
}

/// (term, value)
pub type Entry = (Metadata, Data);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Data {
    NodeInstance(NodeInstance),
    Message(String),
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Metadata {
    pub term: u64,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum Conflict {
    TermConflict(u64), // confilicting term
    LogConflict(u64),  // conflicting index
}
