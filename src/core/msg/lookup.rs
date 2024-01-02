
use crate::id::Id;
use crate::nodeinfo::NodeInfo;

#[allow(dead_code)]
pub(super) struct Filters {
    pub(super) target: Id,
    pub(super) want4: bool,
    pub(super) want6: bool,
    pub(super) want_token: bool
}

impl Filters {
    pub(super) fn new() -> Self {
        Filters {
            target: Id::new(),
            want4: false,
            want6: false,
            want_token: false
        }
    }
}

pub(super) struct Results {
    pub(super) nodes4: Vec<NodeInfo>,
    pub(super) nodes6: Vec<NodeInfo>,
    pub(super) token: i32
}

impl Results {
    pub(super) fn new() -> Self {
        Results {
            nodes4: Vec::new(),
            nodes6: Vec::new(),
            token: 0
        }
    }
}

pub(super) trait FilterProxy  { // for Filters
    fn target(&self) -> &Id;
    fn does_want4(&self) -> bool;
    fn does_want6(&self) -> bool;

    fn set_want4(&mut self);
    fn set_want6(&mut self);
 }

 pub(super) trait ResultProxy { // for Results
    fn nodes4(&self) -> &[NodeInfo];
    fn nodes6(&self) -> &[NodeInfo];
    fn token(&self) -> i32;

    fn set_nodes4(&self, nodes: &[NodeInfo]);
    fn set_nodes6(&self, nodes: &[NodeInfo]);
    fn set_token(&self, token: i32);
 }
