use std::rc::Rc;
use std::collections::LinkedList;
use crate::id::Id;
use crate::dht::DHT;
use crate::kbucket::KBucket;
use crate::kbucket_entry::KBucketEntry;
use crate::node::Node;

pub(crate) struct RoutingTable {
    dht: Option<Rc<DHT>>,
    buckets: LinkedList<Box<KBucket>>
}

#[allow(dead_code)]
impl RoutingTable {
    pub(crate) fn new() -> Self {
        RoutingTable {
            dht: None,
            buckets: LinkedList::new(),
        }
    }

    pub(crate) fn binding_dht(&mut self, dht: Rc<DHT>) -> &mut Self {
        self.dht = Some(dht); self
    }

    fn dht(&self) -> Rc<DHT> {
        self.dht.as_ref().unwrap().clone()
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
        self.buckets.iter().for_each (|item| {
            len += item.size()
        });
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
