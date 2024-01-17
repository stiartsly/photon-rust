use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use boson::id::Id;
use boson::node::{Node, NodeInfo};

#[cfg(test)]
mod apitests {
    use super::*;

    #[test]
    fn test_new_using_ipv4() {
        let id = Id::random();
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 12345);
        let node = Node::new(&id, &addr);
        assert_eq!(node.id(), &id);
        assert_eq!(node.socket_addr(), &addr);
        assert_eq!(node.ip(), Ipv4Addr::new(127,0,0,1));
        assert_eq!(node.port(), 12345);
        assert_eq!(node.version(), 0);
        assert_eq!(node.is_ipv4(), true);
    }

    #[test]
    fn test_new_using_ipv6() {
        let id = Id::random();
        let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0,0,0,0,0,0,0,1)), 12345);
        let node = Node::new(&id, &addr);
        assert_eq!(node.id(), &id);
        assert_eq!(node.socket_addr(), &addr);
        assert_eq!(node.ip(), IpAddr::V6(Ipv6Addr::new(0,0,0,0,0,0,0,1)));
        assert_eq!(node.port(), 12345);
        assert_eq!(node.version(), 0);
        assert_eq!(node.is_ipv6(), true);
    }

    #[test]
    fn test_equal() {
        let id = Id::random();
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 12345);
        let node1 = Node::new(&id, &addr);
        let node2 = Node::new(&id, &addr);
        assert_eq!(node1, node2);
        assert_eq!(node1.id(), node2.id());
        assert_eq!(node1.socket_addr(), node2.socket_addr());
        assert_eq!(node1.version(), node2.version());
    }

    #[test]
    fn test_version() {
        let id = Id::random();
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 12345);
        let mut node = Node::new(&id, &addr);
        node.with_version(5);
        assert_eq!(node.id(), &id);
        assert_eq!(node.socket_addr(), &addr);
        assert_eq!(node.version(), 5);
    }

}
