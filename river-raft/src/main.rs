use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use node::Node;

pub mod logstore;
pub mod node;

pub fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
