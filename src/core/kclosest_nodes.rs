use std::rc::Rc;
use std::cell::RefCell;
use std::cmp::Ordering;

use crate::{
    unwrap,
    id::Id,
    node_info::NodeInfo,
    kbucket::KBucket,
    kbucket_entry::KBucketEntry,
    routing_table::RoutingTable
};

pub(crate) struct KClosestNodes<'a> {
    target: &'a Id,
    routing_table: Rc<RefCell<RoutingTable>>,

    entries: Vec<Box<KBucketEntry>>,
    capacity: usize,

    filter: Box<dyn Fn(&Box<KBucketEntry>) -> bool>,
}

#[allow(dead_code)]
impl<'a> KClosestNodes<'a> {
    pub(crate) fn new(target: &'a Id,
        routing_table: Rc<RefCell<RoutingTable>>,
        max_entries: usize
    ) -> Self {

        Self {
            target,
            routing_table,
            entries: Vec::new(),
            capacity: max_entries,
            filter: Box::new(|entry| entry.is_eligible_for_nodes_list()),
        }

    }

    pub(crate) fn new_with_filter<F>(target: &'a Id,
        routing_table: Rc<RefCell<RoutingTable>>,
        max_entries: usize,
        filter: F
    ) -> Self where F: Fn(&Box<KBucketEntry>) -> bool + 'static {

        Self {
            target,
            routing_table,
            entries: Vec::new(),
            capacity: max_entries,
            filter: Box::new(filter),
        }
    }

    pub(crate) const fn target(&self) -> &Id {
        &self.target
    }

    pub(crate) fn size(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn fill(&mut self, include_itself: bool) {
        let mut idx = 0;
        let mut bucket = None;
        let cloned_routing_table = Rc::clone(&self.routing_table);
        let binding_routing_table = cloned_routing_table.borrow();

        for (k,v) in binding_routing_table.buckets().iter() {
            if self.target > k {
                bucket = Some(v);
                break;
            }
            idx += 1;
        }
        self.insert_entries(bucket);

        let mut low  = idx;
        let mut high = idx;
        let mut iter = binding_routing_table.buckets().iter();
        while self.entries.len() < self.capacity {
            let mut low_bucket  = None;
            let mut high_bucket = None;

            if low > 0 {
                low_bucket = iter.nth(low);
            }
            if high < iter.len() {
                high_bucket = iter.nth(high);
            }

            if low_bucket.is_none() && high_bucket.is_none() {
                break;
            } else if let None = low_bucket {
                high += 1;
                self.insert_entries(high_bucket.map(|(_,v)|v));
            } else if let None = high_bucket {
                low -= 1;
                self.insert_entries(low_bucket.map(|(_,v)|v));
            } else {
                let ordering = self.target.three_way_compare(
                    &unwrap!(low_bucket).1.prefix().last(),
                    &unwrap!(high_bucket).1.prefix().first()
                );
                match ordering {
                    Ordering::Less => {
                        low -= 1;
                        self.insert_entries(low_bucket.map(|(_,v)|v));
                    },
                    Ordering::Greater => {
                        high += 1;
                        self.insert_entries(high_bucket.map(|(_,v)|v));
                    },
                    Ordering::Equal => {
                        low -= 1;
                        high += 1;
                        self.insert_entries(low_bucket.map(|(_,v)|v));
                        self.insert_entries(high_bucket.map(|(_,v)|v));

                    }
                }
            }
        }

        if self.entries.len() < self.capacity {
            // TODO: bootstraps.
        }

        if self.entries.len() < self.capacity && include_itself {
            let bucket_entry = Box::new(KBucketEntry::new(
                binding_routing_table.node_id(),
                binding_routing_table.node_addr()
            ));
            self.entries.push(bucket_entry);
        }

        self.shave();
    }

    pub(crate) fn full(&self) -> bool {
        self.entries.len() >= self.capacity
    }

    fn insert_entries(&mut self, bucket: Option<&Box<KBucket>>) {
        if let Some(v) = bucket {
            v.entries().iter().for_each(|(_,item)| {
                if (self.filter)(item) {
                    self.entries.push(item.clone())
                }
            })
        }
    }

    fn shave(&mut self) {
        self.entries.dedup();

        if self.entries.len() <= self.capacity {
            return;
        }

        self.entries.sort_by(|cn1, cn2|
            self.target.three_way_compare(cn1.node_id(), cn2.node_id())
        );
        _ = self.entries.split_off(self.capacity);
        // Here obsolete list resource would be freed along with
        // all kbucketEntry inside.
    }

    pub(crate) fn as_nodes(&self) -> Vec<NodeInfo> {
        self.entries.iter().map(|x| x.node().clone()).collect()
    }
}
