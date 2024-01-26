use std::rc::Rc;
use std::vec::Vec;
use std::collections::LinkedList;

use crate::id::Id;
use crate::node::Node;
use crate::dht::DHT;
use crate::kbucket_entry::KBucketEntry;
use crate::kbucket::KBucket;

#[allow(dead_code)]
pub(crate) struct KClosestNodes<'a> {
    dht: Rc<&'a DHT>,
    target: &'a Id,

    entries: LinkedList<Box<KBucketEntry>>,
    max_entries: usize,

    filter: Box<dyn Fn(&Box<KBucketEntry>) -> bool>
}

#[allow(dead_code)]
impl<'a> KClosestNodes<'a> {
    pub(crate) fn new(dht: Rc<&'a DHT>,
        target: &'a Id,
        max_entries: usize
    ) -> Self {
        KClosestNodes {
            dht,
            target,
            entries: LinkedList::new(),
            max_entries,
            filter: Box::new(|_| true)
        }
    }

    pub(crate) fn with_filter<F>(dht: Rc<&'a DHT>,
        target: &'a Id,
        max_entries: usize,
        filter: F
    ) -> Self
    where F: Fn(&Box<KBucketEntry>) -> bool + 'static{
        KClosestNodes {
            dht,
            target,
            entries: LinkedList::new(),
            max_entries,
            filter: Box::new(filter)
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
        bucket.entries().iter().for_each(|item|  {
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

    pub(crate) fn as_nodes(&self) -> Vec<Node> {
        self.entries.iter()
            .map(|x| x.node().clone())
            .collect()
    }
}
