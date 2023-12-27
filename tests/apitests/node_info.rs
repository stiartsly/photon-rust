use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use boson::id::Id;
use boson::node_info::NodeInfo;

#[cfg(test)]
mod apitests {
    use super::*;

    #[test]
    fn test_new1() {
        let id = Id::zero();
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 12345);
        let node = NodeInfo::new(&id, &addr);
        assert_eq!(node.ip(), Ipv4Addr::new(127,0,0,1));
        assert_eq!(node.port(), 12345);
        assert_eq!(node.version(), 0);
    }

    #[test]
    fn test_new2() {
        let id = Id::zero(); //::1
        let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0,0,0,0,0,0,0,1)), 12345);
        let node = NodeInfo::new(&id, &addr);
        assert_eq!(node.ip(), IpAddr::V6(Ipv6Addr::new(0,0,0,0,0,0,0,1)));
        assert_eq!(node.port(), 12345);
        assert_eq!(node.version(), 0);
    }
}
