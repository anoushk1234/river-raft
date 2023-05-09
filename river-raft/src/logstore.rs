use indexmap::{self, IndexMap};

use serde_derive::{Deserialize, Serialize};

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
}
#[derive(Serialize, Deserialize)]
pub enum Protocol {
    AppendEntryRequest(AppendEntryRequest),
    AppendEntryResponse(AppendEntryResponse),
    RequestVote(RequestVote),
    RespondVote(RespondVote),
    Ping(u64),
    Pong(u64),
}

#[derive(Serialize, Deserialize)]
pub struct AppendEntryRequest {
    pub leader_id: u64,
    pub term: u64,
    pub entry: Entry,
}
#[derive(Serialize, Deserialize)]
pub struct AppendEntryResponse {
    pub term: u64,
    pub result: bool,
    pub conflict: Option<Conflict>,
}
#[derive(Serialize, Deserialize)]
pub struct RequestVote {
    pub term: u64,
    pub candidate_id: u64,
}
#[derive(Serialize, Deserialize)]
pub struct RespondVote {
    pub term: u64,
    pub vote: bool,
}

#[derive(Serialize, Deserialize)]
pub struct NodeInstance {
    pub ip: std::net::SocketAddr,
    pub id: u64,
}

/// (term, value)
pub type Entry = (Metadata, Data);

#[derive(Serialize, Deserialize)]
pub enum Data {
    NodeInstance(NodeInstance),
    Message(String),
}
#[derive(Default, Serialize, Deserialize)]
pub struct Metadata {
    pub term: u64,
}
#[derive(Serialize, Deserialize)]
pub enum Conflict {
    TermConflict(u64), // confilicting term
    LogConflict(u64),  // conflicting index
}
