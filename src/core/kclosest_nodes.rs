use std::vec::Vec;
use std::collections::LinkedList;

use crate::{
    id::Id,
    node_info::NodeInfo,
    kbucket::KBucket,
    kbucket_entry::KBucketEntry
};

pub(crate) struct KClosestNodes<'a> {
    target: &'a Id,

    entries: LinkedList<Box<KBucketEntry>>,
    max_entries: usize,

    filter: Box<dyn Fn(&Box<KBucketEntry>) -> bool>,
}

#[allow(dead_code)]
impl<'a> KClosestNodes<'a> {
    pub(crate) fn new(target: &'a Id, max_entries: usize) -> Self {
        Self {
            target,
            entries: LinkedList::new(),
            max_entries,
            filter: Box::new(|entry| {
                entry.is_eligible_for_nodes_list()
            })
        }
    }

    pub(crate) fn with_filter<F>(target: &'a Id,  max_entries: usize, filter: F) -> Self
    where F: Fn(&Box<KBucketEntry>) -> bool + 'static,
    {
        Self {
            target,
            entries: LinkedList::new(),
            max_entries,
            filter: Box::new(filter),
        }
    }

    pub(crate) const fn target(&self) -> &Id {
        &self.target
    }

    pub(crate) fn size(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn fill(&mut self, _: bool) -> &Self {
        unimplemented!()
    }

    pub(crate) fn full(&self) -> bool {
        self.entries.len() >= self.max_entries
    }

    fn insert_entries(&mut self, bucket: &Box<KBucket>) {
        bucket.entries().iter().for_each(|item| {
            if (self.filter)(item) {
                self.entries.push_back(item.clone())
            }
        })
    }

    fn shave(&self) {
        let overshoot = self.entries.len() - self.max_entries;
        if overshoot <= 0 {
            return;
        }

        /*self.entries.sort_by(|a, b| {
            self.target.three_way_compare(a.id(), b.id())
        }); */

        unimplemented!()
    }

    pub(crate) fn as_nodes(&self) -> Vec<NodeInfo> {
        self.entries.iter().map(|x| x.node().clone()).collect()
    }
}
