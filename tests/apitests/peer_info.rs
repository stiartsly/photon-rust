
use boson::id::Id;
use boson::peer_info::PeerInfo;

#[cfg(test)]
mod apitests {
    use super::*;

    #[test]
    fn test_new() {
        let id = Id::random();
        let port: u16 = 12345;
        let peer = PeerInfo::new(&id, port);
        assert_eq!(peer.id(), &id);
        assert_eq!(peer.has_private_key(), false);
        assert_eq!(peer.node_id(), &id);
        assert_eq!(peer.origin(), &id);
        assert_eq!(peer.port(), port);
        assert_eq!(peer.has_alternative_url(), false);
    }
}
