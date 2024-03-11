use std::net::SocketAddr;
use std::cmp::Ordering;
use std::vec::Vec;
use std::collections::HashSet;

use crate::id::{distance, Id};
use crate::node_info::NodeInfo;
use super::candidate_node::CandidateNode;

#[derive(Clone)]
pub(crate) struct ClosestCandidates {
    target: Id,
    capacity: usize,
    dedup_ids: HashSet<Id>,
    dedup_addrs: HashSet<SocketAddr>,
    closest: Vec<Box<CandidateNode>>,
}

#[allow(dead_code)]
impl ClosestCandidates {
    pub(crate) fn new(target: &Id, capacity: usize) -> Self {
        Self {
            target: target.clone(),
            capacity,
            dedup_ids: HashSet::new(),
            dedup_addrs: HashSet::new(),
            closest: Vec::new(),
        }
    }

    fn reached_capacity(&self) -> bool {
        self.closest.len() >= self.capacity
    }

    fn size(&self) -> usize {
        self.closest.len()
    }

    pub(crate) fn get(&self, id: &Id) -> Option<&Box<CandidateNode>> {
        let mut cn = None;
        for item in self.closest.iter() {
            if item.nodeid() == id {
                cn = Some(item);
                break;
            }
        }
        cn
    }

    pub(crate) fn remove(&mut self, id: &Id) -> Option<Box<CandidateNode>> {
        let mut pos: usize = 0;
        for item in self.closest.iter() {
            if item.nodeid() == id {
                break;
            }
            pos += 1;
        }

        let mut removed = None;
        if pos < self.closest.len() {
            let mut splitted = self.closest.split_off(pos);
            removed = splitted.pop();
            self.closest.append(&mut splitted);
        }
        removed
    }

    pub(crate) fn next(&self) -> Option<&Box<CandidateNode>> {
        let mut cns = Vec::with_capacity(self.closest.len());
        self.closest.iter().for_each(|item| {
            if item.is_eligible() {
                cns.push(item);
            }
        });

        cns.sort_by(|cn1, cn2| self.candidate_order(cn1,cn2));
        cns.pop()
    }

    pub(crate) fn head(&self) -> Id {
        match self.closest.is_empty() {
            true => distance(&self.target, &Id::max()),
            false => self.closest.iter().next().unwrap().nodeid().clone()
        }
    }

    pub(crate) fn tail(&self) -> Id {
        match self.closest.is_empty() {
            true => distance(&self.target, &Id::max()),
            false => self.closest.iter().last().unwrap().nodeid().clone(),
        }
    }

    pub(crate) fn add(&mut self, candidates: &[NodeInfo]) {
        let mut filtered = Vec::new();
        for item in candidates.iter() {
            if !self.dedup_ids.insert(item.id().clone()) ||
                !self.dedup_addrs.insert(item.socket_addr().clone()) {
                continue;
            }
            filtered.push(Box::new(CandidateNode::new(item, false)));
        }

        filtered.sort_by(|cn1, cn2|
            self.target.three_way_compare(cn1.nodeid(), cn2.nodeid())
        );

        self.closest.append(&mut filtered);
        if self.closest.len() >= self.capacity {
            let mut to_remove = Vec::new();
            self.closest.iter().for_each(|item| {
                if !item.is_inflight() {
                    to_remove.push(item);
                }
            });

            if to_remove.len() > self.capacity {
                to_remove.sort_by(|cn1, cn2| self.candidate_order(cn1, cn2));
                while to_remove.len() > self.capacity {
                    _ = to_remove.pop()
                }
            }
        }
    }

    fn candidate_order(&self, a: &CandidateNode, b: &CandidateNode) -> Ordering {
        match a.pinged().cmp(&b.pinged()) {
            Ordering::Equal => self.target.three_way_compare(a.nodeid(), b.nodeid()),
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        }
    }
}
