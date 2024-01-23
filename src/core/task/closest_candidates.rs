use std::boxed::Box;

use crate::id::Id;
use super::candidate_node::CandidateNode;

#[allow(dead_code)]
pub(crate) struct ClosestCandidates {
    target: Id,
    capacity: usize
}

impl ClosestCandidates {
    pub(crate) fn new(target: &Id, capacity: usize) -> Self {
        ClosestCandidates {
            target: target.clone(),
            capacity
        }
    }

    pub(crate) fn remove(&mut self, _: &Id) -> Option<Box<CandidateNode>> {
        unimplemented!()
    }
}
