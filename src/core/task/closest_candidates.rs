use std::rc::Rc;
use std::cell::RefCell;
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
    closest: Vec<Rc<RefCell<CandidateNode>>>,
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

    pub(crate) fn len(&self) -> usize {
        self.closest.len()
    }

    pub(crate) fn get(&self, id: &Id) -> Option<Rc<RefCell<CandidateNode>>> {
        let mut cn = None;
        for item in self.closest.iter() {
            if item.borrow().nodeid() == id {
                cn = Some(Rc::clone(&item));
                break;
            }
        }
        cn
    }

    pub(crate) fn remove(&mut self, id: &Id) -> Option<Rc<RefCell<CandidateNode>>> {
        let mut pos: usize = 0;
        for item in self.closest.iter() {
            if item.borrow().nodeid() == id {
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

    pub(crate) fn next(&mut self) -> Option<Rc<RefCell<CandidateNode>>> {
        let mut cns = Vec::with_capacity(self.closest.len());
        self.closest.iter().for_each(|item| {
            if item.borrow().is_eligible() {
                cns.push(Rc::clone(&item));
            }
        });

        cns.sort_by(|cn1, cn2| self.candidate_order(cn1,cn2));
        cns.pop()
    }

    pub(crate) fn head(&self) -> Id {
        match self.closest.is_empty() {
            true => distance(&self.target, &Id::max()),
            false => self.closest.iter().next().unwrap().borrow().nodeid().clone()
        }
    }

    pub(crate) fn tail(&self) -> Id {
        match self.closest.is_empty() {
            true => distance(&self.target, &Id::max()),
            false => self.closest.iter().last().unwrap().borrow().nodeid().clone(),
        }
    }

    pub(crate) fn add(&mut self, candidates: &[Rc<NodeInfo>]) {
        let mut filtered = Vec::new();
        for item in candidates.iter() {
            if !self.dedup_ids.insert(item.id().clone()) ||
                !self.dedup_addrs.insert(item.socket_addr().clone()) {
                continue;
            }

            filtered.push(
                Rc::new(RefCell::new(CandidateNode::new(item, false)))
            );
        }

        filtered.sort_by(|cn1, cn2|
            self.target.three_way_compare(cn1.borrow().nodeid(), cn2.borrow().nodeid())
        );

        self.closest.append(&mut filtered);
        if self.closest.len() >= self.capacity {
            let mut to_remove = Vec::new();
            self.closest.iter().for_each(|item| {
                if !item.borrow().is_inflight() {
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

    fn candidate_order(&self,
        a: &Rc<RefCell<CandidateNode>>,
        b: &Rc<RefCell<CandidateNode>>) -> Ordering
    {
        match a.borrow().pinged().cmp(&b.borrow().pinged()) {
            Ordering::Equal => {
                self.target.three_way_compare(a.borrow().nodeid(), b.borrow().nodeid())
            },
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        }
    }
}
