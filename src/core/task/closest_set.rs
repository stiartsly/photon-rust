use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::id::{distance, Id};
use super::candidate_node::CandidateNode;

pub(crate) struct ClosestSet {
    target: Id,
    capacity: usize,

    closest: HashMap<Id, Rc<RefCell<CandidateNode>>>,

    insert_attempt_since_tail_modification: usize,
    insert_attempt_since_head_modification: usize,
}

#[allow(dead_code)]
impl ClosestSet {
    pub(crate) fn new(target: &Id, capacity: usize) -> Self {
        Self {
            target: target.clone(),
            capacity,
            closest: HashMap::new(),
            insert_attempt_since_tail_modification: 0,
            insert_attempt_since_head_modification: 0,
        }
    }

    pub(crate) fn reached_capacity(&self) -> bool {
        self.closest.len() >= self.capacity
    }

    pub(crate) fn size(&self) -> usize {
        self.closest.len()
    }

    pub(crate) fn candidate_node(&self, id: &Id) -> Option<Rc<RefCell<CandidateNode>>> {
        self.closest.get(id).map(|item | Rc::clone(&item))
    }



    pub(crate) fn contains(&self, id: &Id) -> bool {
        self.closest.get(id).is_some()
    }

    pub(crate) fn add(&mut self, candidate: Rc<RefCell<CandidateNode>>) {
        let nodeid = candidate.borrow().nodeid().clone();
        self.closest.insert(
            nodeid.clone(),
            candidate
        );

        if self.closest.len() > self.capacity {
            let mut to_remove = None;
            let last = self.closest.iter().last();
            if let Some(item) = last {
                if item.0 == &nodeid {
                    self.insert_attempt_since_tail_modification += 1;
                } else {
                    self.insert_attempt_since_tail_modification = 0;
                }
                to_remove = Some(item.0.clone());
            }
            if let Some(id) = to_remove {
                self.closest.remove(&id);
            }
        }
        let head = self.closest.iter().next();
        if let Some(item) = head {
            if item.0 == &nodeid {
                self.insert_attempt_since_head_modification = 0;
            } else {
                self.insert_attempt_since_head_modification += 1;
            }
        }
    }

    pub(crate) fn remove(&mut self, candidate: &Id) {
        _ = self.closest.remove(candidate)
    }

    pub(crate) fn tail(&self) -> Id {
        match self.closest.is_empty() {
            true => distance(&self.target, &Id::max()),
            false => self.closest.iter().last().unwrap().0.clone(),
        }
    }

    pub(crate) fn head(&self) -> Id {
        match self.closest.is_empty() {
            true => distance(&self.target, &Id::max()),
            false => self.closest.iter().next().unwrap().0.clone(),
        }
    }

    pub(crate) fn is_eligible(&self) -> bool {
        self.reached_capacity() &&
            self.insert_attempt_since_tail_modification > self.capacity
    }
}
