use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use logstore::{LogStore, NodeInstance};
use node::Node;
use parking_lot::RwLock;

pub mod logstore;
pub mod node;

pub fn main() {
    let entrypoint = vec![
        NodeInstance {
            sender_ip: "127.0.0.1:6585".parse().unwrap(),
            receiver_ip: "127.0.0.1:6586".parse().unwrap(),
            id: 1,
        },
        NodeInstance {
            sender_ip: "127.0.0.1:6587".parse().unwrap(),
            receiver_ip: "127.0.0.1:6588".parse().unwrap(),
            id: 2,
        },
    ];
    let log_store = Arc::new(RwLock::new(LogStore::create_entrypoint(entrypoint)));
    let mut node1 = Node::init(1, "127.0.0.1:6585", "127.0.0.1:6586", log_store.clone());
    let node2 = Node::init(2, "127.0.0.1:6587", "127.0.0.1:6588", log_store);
    // let n1 = node1.clone();
    std::thread::spawn(move || loop {
        node2.clone().start();
    });
    // node1.start();

    node1.start();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
