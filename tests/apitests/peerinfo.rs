
use boson::id::Id;
use boson::peerinfo::PeerInfo;

#[cfg(test)]
mod apitests {
    use super::*;

    #[test]
    fn test_new() {
        let id = Id::random();
        let port: u16 = 12345;
        let peer = PeerInfo::new(&id, port).map_err(|_| "failed").unwrap();

        assert_eq!(peer.has_private_key(), true);
        assert_eq!(peer.node_id(), &id);
        assert_eq!(peer.origin(), &id);
        assert_eq!(peer.port(), port);
        assert_eq!(peer.has_alternative_url(), false);
        assert_eq!(peer.is_delegated(), false);
    }
}
