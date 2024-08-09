use std::rc::Rc;
use std::collections::BTreeMap;
use std::time::SystemTime;

use crate::{
    Id,
    Prefix,
    NodeInfo,
    node_info::Reachable,
    kbucket::KBucket,
    kbucket_entry::KBucketEntry
};

#[allow(dead_code)]
pub(crate) struct RoutingTable {
    nodeid: Rc<Id>,
    buckets: BTreeMap<Id, Box<KBucket>>,

    last_of_last_ping_check: SystemTime,

}

#[allow(dead_code)]
impl RoutingTable {
    pub(crate) fn new(nodeid: Rc<Id>) -> Self {
        let prefix = Prefix::new();
        let mut buckets = BTreeMap::new();

        buckets.insert(
            prefix.id().clone(),
            Box::new(KBucket::new(prefix, true))
        );

        Self {
            nodeid,
            buckets,
            last_of_last_ping_check: SystemTime::UNIX_EPOCH,
        }
    }

    pub(crate) fn buckets(&self) -> &BTreeMap<Id, Box<KBucket>> {
        &self.buckets
    }

    fn bucket_mut(&mut self, target: &Id) -> Option<&mut Box<KBucket>> {
        self.buckets.iter_mut()
            .find(|(k, _)| target >= k)
            .map(|(_, v)| v)
    }

    fn bucket(&self, target: &Id) -> Option<&Box<KBucket>> {
        self.buckets.iter()
            .find(|(k, _)| target >= k)
            .map(|(_, v)| v)
    }

    pub(crate) fn bucket_entry(&self, id: &Id) -> Option<&Box<KBucketEntry>> {
        self.bucket(id).and_then(|bucket| bucket.entry(id))
    }

    fn pop_bucket(&mut self, target: &Id) -> Option<Box<KBucket>> {
        let key = self.buckets.keys()
            .find(|&k| target >= k)
            .cloned();

        key.and_then(|k| self.buckets.remove(&k))
    }

    pub(crate) fn size(&self) -> usize {
        self.buckets.len()
    }

    pub(crate) fn random_entry(&self) ->Option<&Box<NodeInfo>> {
        // TODO: unimplemented!()
        None
    }

    pub(crate) fn random_entries(&self, _: i32) -> Option<Vec<Box<NodeInfo>>> {
        // TODO: unimplemented!()
        Some(Vec::new())
    }

    pub(crate) fn random_node(&self) -> Option<Rc<NodeInfo>> {
        // TODO:
        None
    }

    pub(crate) fn random_nodes(&self, _: i32) -> Option<Vec<Rc<NodeInfo>>> {
        // TODO:
        Some(Vec::new())
    }

    fn is_home_bucket(&self, prefix: &Prefix) -> bool {
        prefix.is_prefix_of(self.nodeid.as_ref())
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
            prefix.is_prefix_of(self.nodeid.as_ref())
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
    if  !bucket.prefix().is_splittable() ||
        !bucket.is_full() ||
        !new_entry.reachable() ||
        bucket.exists(new_entry.node_id()) ||
        bucket.needs_replacement() {

        return false
    }
    let high = bucket.prefix().split_branch(true);
    high.is_prefix_of(new_entry.node_id())
}
