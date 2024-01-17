use std::rc::Rc;
use std::collections::LinkedList;
use crate::id::Id;
use crate::dht::DHT;
use crate::kbucket::KBucket;
use crate::kbucket_entry::KBucketEntry;
use crate::node::Node;

#[allow(dead_code)]
pub(crate) struct RoutingTable {
    dht: Rc<Box<DHT>>,

    buckets: LinkedList<Box<KBucket>>
}

#[allow(dead_code)]
impl RoutingTable {
    pub(crate) fn new(dht: Box<DHT>) -> Self {
        RoutingTable {
            dht: Rc::new(dht),
            buckets: LinkedList::new(),
        }
    }

    pub(crate) fn kbucket(&self, _: &Id) -> Option<&KBucket> {
        unimplemented!();
    }

    pub(crate) fn kentry(&self, id: &Id) -> Option<&KBucketEntry> {
        match self.kbucket(id) {
            Some(bucket) => { bucket.kentry(id) },
            None => None,
        }
    }

    pub(crate) fn random_entry(&self) -> Node {
        unimplemented!()
    }

    pub(crate) fn random_entries(&self, _: i32) -> Vec<Node> {
        unimplemented!();
    }

    pub(crate) fn put(&self, _: Box<KBucket>) {
        unimplemented!();
    }

    pub(crate) fn remove(&self, _: &Id) {
        unimplemented!();
    }

    pub(crate) fn on_send(&self, _: &Id) {
        unimplemented!();
    }

    pub(crate) fn on_timeout(&self, _: &Id) {
        unimplemented!();
    }

    pub(crate) fn maintenance(&self) {
        unimplemented!();
    }

    pub(crate) fn load(&self, _: &str) {
        unimplemented!();
    }

    pub(crate) fn save(&self, _: &str) {
        unimplemented!();
    }
}

