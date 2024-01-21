use std::collections::HashMap;

use crate::id::Id;
use super::candidate_node::CandidateNode;

#[allow(dead_code)]
pub(crate) struct ClosestSet<'a> {
    target: &'a Id,
    capacity: usize,

    closest: HashMap<Id, Box<CandidateNode>>,

    insert_attempt_since_tail_modification: usize,
    insert_attempt_since_head_modification: usize
}

#[allow(dead_code)]
impl<'a> ClosestSet<'a> {
    pub(crate) fn new(target: &'a Id, capacity: usize) -> Self {
        ClosestSet {
            target,
            capacity,
            closest: HashMap::new(),
            insert_attempt_since_tail_modification: 0,
            insert_attempt_since_head_modification: 0,
        }
    }

    pub(crate) fn reach_capacity(&self) -> bool {
        self.closest.len() >= self.capacity
    }

    pub(crate) fn size(&self) -> usize {
        self.closest.len()
    }

    pub(crate) fn candidate_node(&self, id: &Id) -> Option<&Box<CandidateNode>> {
        self.closest.get(id)
    }

    pub(crate) fn contains(&self, id: &Id) -> bool {
        self.closest.get(id).is_some()
    }

    pub(crate) fn add(&self, _: &Box<CandidateNode>) {
        unimplemented!()
    }

    pub(crate) fn remove(&mut self, candidate: &Id) {
        _ = self.closest.remove(candidate)
    }

    pub(crate) fn tail(&self) -> Id {
        unimplemented!()
    }

    pub(crate) fn head(&self) -> Id {
        unimplemented!()
    }

    pub(crate) fn is_eligible(&self) -> bool {
        self.reach_capacity() && self.insert_attempt_since_tail_modification > self.capacity
    }
}
