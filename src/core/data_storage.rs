use std::time::SystemTime;

use crate::id::Id;
use crate::peer::Peer;
use crate::value::Value;
use crate::error::Error;

pub(crate) trait DataStorage {
    fn open(&mut self, _: &str) -> Result<(), Error>;

    fn get_value(&self, _: &Id) -> Option<Box<Value>>;
    fn remove_value(&mut self, _: &Id) -> bool;

    fn put_value_and_update(
        &mut self,
        expected_seq: i32,
        persistent: bool,
        update_last_announce: bool,
    );
    fn put_value(&mut self, _: Box<Value>, persistent: bool) {
        self.put_value_and_update(-1, persistent, true);
    }

    fn update_value_last_announce(&mut self, value_id: &Id);
    fn get_persistent_values(&self, last_announce_before: &SystemTime) -> Vec<Box<Value>>;
    fn get_all_values(&self) -> Vec<Box<Value>>;

    fn get_peers(&self, peer_id: &Id, max_peers: i32) -> Vec<Box<Peer>>;
    fn get_peer(&self, peer_id: &Id, origin: &Id) -> Box<Peer>;
    fn remove_peer(&mut self, peer_id: &Id, origin: &Id) -> bool;

    fn put_peers(&mut self, _: &[Box<Peer>]);
    fn put_peer(&mut self, _: Box<Peer>, persistent: bool, update_last_announce: bool);

    fn put_peer1(&mut self, peer: Box<Peer>, persistent: bool) {
        self.put_peer(peer, persistent, true);
    }

    fn put_peer2(&mut self, peer: Box<Peer>) {
        self.put_peer(peer, false, false);
    }

    fn update_peer_last_announce(&mut self, peer_id: &Id, origin: &Id);
    fn get_perisistent_peers(&mut self, last_annouce_before: bool);
    fn get_all_peers(&mut self) -> Vec<Box<Id>>;

    fn close(&mut self);
}
