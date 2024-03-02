use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;

use crate::{dht::DHT, id::Id, kbucket::KBucket, kbucket_entry::KBucketEntry, node::Node};

pub(crate) struct RoutingTable {
    dht: Option<Rc<RefCell<DHT>>>,
    buckets: LinkedList<Box<KBucket>>,
}

#[allow(dead_code)]
impl RoutingTable {
    pub(crate) fn new() -> Self {
        RoutingTable {
            dht: None,
            buckets: LinkedList::new(),
        }
    }

    pub(crate) fn bind_dht(&mut self, dht: Rc<RefCell<DHT>>) {
        self.dht = Some(dht)
    }

    pub(crate) fn unbind_dht(&mut self) {
        _ = self.dht.take()
    }

    fn dht(&self) -> &Rc<RefCell<DHT>> {
        assert!(self.dht.is_some(), "not dht bound");
        self.dht.as_ref().unwrap()
    }

    fn bucket_mut(&self, _: &Id) -> &mut Box<KBucket> {
        unimplemented!()
    }

    pub(crate) fn bucket(&self, _: &Id) -> &Box<KBucket> {
        unimplemented!()
    }

    pub(crate) fn bucket_entry(&self, id: &Id) -> Option<&Box<KBucketEntry>> {
        self.bucket(id).entry(id)
    }

    pub(crate) fn size(&self) -> usize {
        let mut len: usize = 0;
        self.buckets.iter().for_each(|item| len += item.size());
        len
    }

    pub(crate) fn random_entry(&self) -> Node {
        unimplemented!()
    }

    pub(crate) fn random_entries(&self, _: i32) -> Vec<Node> {
        unimplemented!();
    }

    pub(crate) fn put(&mut self, _: Box<KBucket>) {
        unimplemented!();
    }

    pub(crate) fn remove(&self, _: &Id) {
        unimplemented!()
    }

    pub(crate) fn on_timeout(&self, id: &Id) {
        self.bucket_mut(id).on_timeout(id)
    }

    pub(crate) fn on_send(&self, id: &Id) {
        self.bucket_mut(id).on_send(id)
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

    fn _put(&mut self, entry: &Box<KBucketEntry>) {
        let id = entry.id();
        let mut bucket = self.bucket_mut(id);

        while _need_split(bucket, &entry) {
            _split(bucket);
            bucket = self.bucket_mut(id);
        }
        bucket._put(entry)
    }
}

fn _need_split(_: &mut Box<KBucket>, _: &Box<KBucketEntry>) -> bool {
    unimplemented!()
}

fn _split(_: &mut Box<KBucket>) {
    unimplemented!()
}
