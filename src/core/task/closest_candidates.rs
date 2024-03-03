use std::boxed::Box;
use std::cmp::Ordering;
use std::collections::LinkedList;
use std::vec::Vec;

use super::candidate_node::CandidateNode;
use crate::id::{distance, Id};
use crate::node_info::NodeInfo;

#[allow(dead_code)]
pub(crate) struct ClosestCandidates {
    target: Id,
    capacity: usize,

    closest: LinkedList<Box<CandidateNode>>,
}

#[allow(dead_code)]
impl ClosestCandidates {
    pub(crate) fn new(target: &Id, capacity: usize) -> Self {
        ClosestCandidates {
            target: target.clone(),
            capacity,
            closest: LinkedList::new(),
        }
    }

    pub(crate) fn get(&self, id: &Id) -> Option<&Box<CandidateNode>> {
        for item in self.closest.iter() {
            if item.node().id() == id {
                return Some(&item);
            }
        }
        None
    }

    pub(crate) fn remove(&mut self, id: &Id) -> Option<Box<CandidateNode>> {
        let mut at: usize = 0;
        for item in self.closest.iter() {
            if item.node().id() != id {
                at += 1;
            }
        }

        if at >= self.closest.len() {
            return None;
        }

        let mut splitted = self.closest.split_off(at);
        let removed = splitted.pop_front();
        self.closest.append(&mut splitted);
        return removed;
    }

    pub(crate) fn next(&mut self) -> Option<&Box<CandidateNode>> {
        let mut candidates: Vec<&Box<CandidateNode>> = Vec::with_capacity(self.closest.len());
        self.closest.iter().for_each(|item| {
            if item.is_eligible() {
                candidates.push(&item);
            }
        });

        if !candidates.is_empty() {
            candidates.sort_by(|a, b| {
                let comparison = a.pinged() - b.pinged();
                if comparison != 0 {
                    return match comparison < 0 {
                        true => Ordering::Less,
                        false => Ordering::Greater,
                    };
                }
                self.target.three_way_compare(a.node().id(), b.node().id())
            });
            return candidates.pop();
        }
        None
    }

    pub(crate) fn head(&self) -> Id {
        match self.closest.is_empty() {
            true => distance(&self.target, &Id::max()),
            false => self.closest.front().unwrap().node().id().clone(),
        }
    }

    pub(crate) fn tail(&self) -> Id {
        match self.closest.is_empty() {
            true => distance(&self.target, &Id::max()),
            false => self.closest.back().unwrap().node().id().clone(),
        }
    }

    pub(crate) fn add(&mut self, _: &[NodeInfo]) {
        unimplemented!()
    }

    fn candidate_order(&self, a: &CandidateNode, b: &CandidateNode) -> Ordering {
        let comparison = a.pinged() - b.pinged();
        if comparison != 0 {
            return match comparison < 0 {
                true => Ordering::Less,
                false => Ordering::Greater,
            };
        }
        self.target.three_way_compare(a.node().id(), b.node().id())
    }
}
