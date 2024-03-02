use std::time::SystemTime;

use crate::node::{Node, Reachable};

#[derive(Clone)]
pub(crate) struct CandidateNode {
    node: Node,

    last_sent: SystemTime,
    last_reply: SystemTime,

    reachable: bool,
    // acked: bool, // TODO: CHECK
    pinged: i32,

    token: i32,
}

#[allow(dead_code)]
impl CandidateNode {
    pub(crate) fn new(node: &Node, reachable: bool) -> Self {
        CandidateNode {
            node: node.clone(),
            last_sent: SystemTime::UNIX_EPOCH,
            last_reply: SystemTime::UNIX_EPOCH,
            reachable,
            // acked: false,
            pinged: 0,
            token: 0,
        }
    }

    pub(crate) fn node(&self) -> &Node {
        &self.node
    }

    pub(crate) fn set_sent(&mut self) {
        self.last_sent = SystemTime::now();
        self.pinged += 1;
    }

    pub(crate) fn clear_sent(&mut self) {
        self.last_sent = SystemTime::UNIX_EPOCH;
    }

    pub(crate) fn pinged(&self) -> i32 {
        self.pinged
    }

    pub(crate) fn set_replied(&mut self) {
        self.last_reply = SystemTime::now();
    }

    pub(crate) fn set_token(&mut self, token: i32) {
        self.token = token
    }

    pub(crate) fn token(&self) -> i32 {
        self.token
    }

    pub(crate) fn is_inflight(&self) -> bool {
        self.last_sent != SystemTime::UNIX_EPOCH
    }

    pub(crate) fn is_eligible(&self) -> bool {
        self.last_sent == SystemTime::UNIX_EPOCH && self.pinged < 3
    }
}

impl Reachable for CandidateNode {
    fn reachable(&self) -> bool {
        self.reachable
    }

    fn unreachable(&self) -> bool {
        self.pinged >= 3
    }

    fn set_reachable(&mut self, reachable: bool) {
        self.reachable = reachable
    }
}
