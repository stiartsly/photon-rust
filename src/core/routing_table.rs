use std::net::SocketAddr;
use std::collections::BTreeMap;

use crate::{
    id::Id,
    prefix::Prefix,
    node_info::{NodeInfo, Reachable},
    kbucket::KBucket,
    kbucket_entry::KBucketEntry
};

pub(crate) struct RoutingTable {
    ni: NodeInfo,
    buckets: BTreeMap<Id, Box<KBucket>>,
}

#[allow(dead_code)]
impl RoutingTable {
    pub(crate) fn new(id: &Id, addr: &SocketAddr) -> Self {
        let prefix = Prefix::new();
        let mut buckets = BTreeMap::new();
        buckets.insert(
            prefix.id().clone(),
            Box::new(KBucket::new(prefix, true))
        );

        Self {
            ni: NodeInfo::new(id, addr),
            buckets,
        }
    }

    pub(crate) fn node_id(&self) -> &Id {
        self.ni.id()
    }

    pub(crate) fn node_addr(&self) -> &SocketAddr {
        self.ni.socket_addr()
    }

    pub(crate) fn buckets(&self) -> &BTreeMap<Id, Box<KBucket>> {
        &self.buckets
    }

    fn bucket_mut<'a>(&'a mut self, target: &Id) -> Option<&'a mut Box<KBucket>> {
        let mut bucket = None;
        let mut iter = self.buckets.iter_mut();
        while let Some((key,val)) = iter.next() {
            bucket = Some(val);
            if target >= key {
                break;
            }
        }
        bucket
    }

    fn pop_bucket(&mut self, target: &Id) -> Option<Box<KBucket>> {
        let mut bucket = None;
        let mut iter = self.buckets.iter();
        while let Some((key,_)) = iter.next() {
            if target >= key {
                bucket = Some(key.clone());
                break;
            }
        }

        match bucket {
            Some(key) => self.buckets.remove(&key),
            None => None
        }
    }

    pub(crate) fn bucket(&self, target: &Id) -> Option<&Box<KBucket>> {
        let mut bucket = None;
        let mut iter = self.buckets.iter();
        while let Some((key, val)) = iter.next() {
            if target >= key {
                bucket = Some(val);
                break;
            }
        }
        bucket
    }

    pub(crate) fn bucket_entry(&self, id: &Id) -> Option<&Box<KBucketEntry>> {
        match self.bucket(id) {
            Some(bucket) => bucket.entry(id),
            None => None
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.buckets.len()
    }

    pub(crate) fn random_entry(&self) -> NodeInfo {
        unimplemented!()
    }

    pub(crate) fn random_entries(&self, _: i32) -> Vec<Box<NodeInfo>> {
        //unimplemented!();
        Vec::new()
    }

    fn is_home_bucket(&self, prefix: &Prefix) -> bool {
        prefix.is_prefix_of(self.ni.id())
    }

    pub(crate) fn put(&mut self, entry: Box<KBucketEntry>) {
        self._put(entry)
    }

    pub(crate) fn remove(&mut self, id: &Id) {
        self._remove(id)
    }

    pub(crate) fn on_timeout(&mut self, id: &Id) {
        self._on_timeout(id)
    }

    pub(crate) fn on_send(&mut self, id: &Id) {
        self._on_send(id)
    }

    pub(crate) fn maintenance(&self) {
        //unimplemented!();
    }

    pub(crate) fn load(&self, _: &str) {
        // unimplemented!();
    }

    pub(crate) fn save(&self, _: &str) {
        // unimplemented!();
    }

    fn _split(&mut self, mut bucket: Box<KBucket>) {
        let prefix = bucket.prefix();
        let pl = prefix.split_branch(false);
        let ph = prefix.split_branch(true);

        let home_bucket = |prefix: &Prefix| -> bool {
            prefix.is_prefix_of(self.node_id())
        };

        let mut low  = Box::new(KBucket::new(pl.clone(), home_bucket(&pl)));
        let mut high = Box::new(KBucket::new(ph.clone(), home_bucket(&ph)));

        while let Some(entry) = bucket.pop() {
            match low.prefix().is_prefix_of(entry.node_id()) {
                true => low._put(entry),
                false => high._put(entry)
            }
        }

        self.buckets.insert(pl.id().clone(), low);
        self.buckets.insert(ph.id().clone(), high);
    }

    fn _put(&mut self, new_entry: Box<KBucketEntry>) {
        let id = new_entry.node_id().clone();

        while let Some(mut bucket) = self.pop_bucket(&id) {
            if _needs_split(&bucket, &new_entry) {
                self._split(bucket);
            } else {
                bucket._put(new_entry);
                self.buckets.insert(id, bucket);
                break;
            }
        }
    }

    fn _remove(&mut self, _: &Id) {
        unimplemented!()
    }

    fn _on_timeout(&mut self, id: &Id) {
        if let Some(v) = self.bucket_mut(id) {
            v.on_timeout(id)
        }
    }

    fn _on_send(&mut self, id: &Id) {
        if let Some(v) = self.bucket_mut(id) {
            v.on_send(id)
        }
    }
}

fn _needs_split(bucket: &Box<KBucket>, new_entry: &Box<KBucketEntry>) -> bool {
    if !bucket.prefix().is_splittable() ||
        !bucket.is_full() ||
        !new_entry.reachable() ||
        bucket.exists(new_entry.node_id()) ||
        bucket.needs_replacement() {

        return false
    }
    let high = bucket.prefix().split_branch(true);
    high.is_prefix_of(new_entry.node_id())
}
