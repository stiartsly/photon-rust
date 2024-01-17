use std::vec::Vec;
use std::collections::LinkedList;

use crate::id::Id;
use crate::node::Node;
use crate::dht::DHT;
use crate::kbucket_entry::KBucketEntry;

#[allow(dead_code)]
pub(crate) struct KClosestNodes<'a> {
    //dht: &'a Box<DHT>,
    dht: &'a DHT,
    target: &'a Id,

    entries: LinkedList<Box<KBucketEntry>>,
    max_entries: usize,

    filter: Box<dyn Fn(&Box<KBucketEntry>) -> bool>
}

#[allow(dead_code)]
impl<'a> KClosestNodes<'a> {
    pub(crate) fn new(dht: &'a DHT, target: &'a Id, max_entries: usize) -> Self {
        KClosestNodes {
            dht,
            target,
            entries: LinkedList::new(),
            max_entries,
            filter: Box::new(|_| true)
        }
    }

    pub(crate) fn with_filter<F>(dht: &'a DHT, target: &'a Id, max_entries: usize, filter: &'static F) -> Self
    where F: Fn(&Box<KBucketEntry>) -> bool {
        KClosestNodes {
            dht,
            target,
            entries: LinkedList::new(),
            max_entries,
            filter: Box::new(filter)
        }
    }

    pub(crate) const fn targget(&self) -> &Id {
        &self.target
    }

    pub(crate) fn size(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn fill(&mut self, _: bool) {
        unimplemented!()
    }

    pub(crate) fn is_full(&self) -> bool {
        self.entries.len() >= self.max_entries
    }

    pub(crate) fn as_nodes(&self) -> Vec<Node> {
        self.entries.iter()
            .map(|x| x.node_info().clone())
            .collect()
    }
}
