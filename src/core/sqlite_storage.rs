use std::time::SystemTime;

use crate::{
    Id,
    Peer,
    Value,
    error::Error,
    data_storage::DataStorage
};

pub(crate) struct SqliteStorage {}

impl SqliteStorage {
    pub(crate) fn new() -> Self {
        SqliteStorage {}
    }
}

impl DataStorage for SqliteStorage {
    fn open(&mut self, _: String) -> Result<(), Error> {
        Ok(())
        //TODO
    }

    fn value(&self, _: &Id) -> Option<Box<Value>> {
        unimplemented!()
    }

    fn remove_value(&mut self, _: &Id) -> bool {
        unimplemented!()
    }

    fn put_value_and_update(&mut self, _: i32, _: bool, _: bool) {
        unimplemented!()
    }

    fn put_value(&mut self, _: Box<Value>, persistent: bool) {
        self.put_value_and_update(-1, persistent, true);
    }

    fn update_value_last_announce(&mut self, _: &Id) {
        unimplemented!()
    }

    fn persistent_values(&self, _: &SystemTime) -> Vec<Box<Value>> {
        unimplemented!()
    }

    fn values(&self) -> Vec<Box<Value>> {
        unimplemented!()
    }

    fn peers(&self, _: &Id, _: i32) -> Vec<Box<Peer>> {
        unimplemented!()
    }

    fn get_peer(&self, _: &Id, _: &Id) -> Box<Peer> {
        unimplemented!()
    }

    fn remove_peer(&mut self, _: &Id, _: &Id) -> bool {
        unimplemented!()
    }

    fn put_peers(&mut self, _: &[Box<Peer>]) {
        unimplemented!()
    }

    fn put_peer(&mut self, _: Box<Peer>, _: bool, _: bool) {
        unimplemented!()
    }

    fn put_peer1(&mut self, peer: Box<Peer>, persistent: bool) {
        self.put_peer(peer, persistent, true);
    }

    fn put_peer2(&mut self, peer: Box<Peer>) {
        self.put_peer(peer, false, false);
    }

    fn update_peer_last_announce(&mut self, _: &Id, _: &Id) {
        unimplemented!()
    }

    fn perisistent_peers(&mut self, _: bool) -> Vec<Box<Peer>>{
        unimplemented!()
    }

    fn peer_ids(&mut self) -> Vec<Box<Id>> {
        unimplemented!()
    }

    fn close(&mut self) {
        // TODO;
    }
}
