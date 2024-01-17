use crate::node::{Node, Visit};

#[allow(dead_code)]
pub(crate) struct CandidateNode {
    nodeinfo: Node,

    last_sent: u64,
    last_reply: u64,

    reachable: bool,
    acked: bool,
    pinged: i32,

    token: i32
}

#[allow(dead_code)]
impl CandidateNode {
    pub(crate) fn new(ni: &Node) -> Self {
        CandidateNode {
            nodeinfo: ni.clone(),
            last_sent: 0,
            last_reply: 0,
            reachable: false, // TODO:
            acked: false,
            pinged: 0,
            token: 0
        }
    }

    pub(crate) fn set_sent(&mut self) {
        self.last_sent = 0; // TODO:
        self.last_reply = 0;
    }

    pub(crate) fn clear_sent(&mut self) {
        self.last_sent = 0
    }

    pub(crate) fn pinged(&self) -> i32 {
        self.pinged
    }

    pub(crate) fn set_replied(&mut self) {
        self.last_reply = 0; // TODO:
    }

    pub(crate) fn token(&self) -> i32 {
        self.token
    }

    pub(crate) fn set_token(&mut self, token: i32) {
        self.token = token
    }

    pub(crate) fn is_inflight(&self) -> bool {
        self.last_sent != 0
    }

    pub(crate) fn is_eligible(&self) -> bool {
        self.last_sent == 0 && self.pinged < 3
    }
}

impl Visit for CandidateNode {
    fn reachable(&self) -> bool {
        self.reachable
    }

    fn unreachable(&self) -> bool {
        self.pinged >= 3
    }

    fn with_reachable(&mut self, reachable: bool) -> &mut Self {
        self.reachable = reachable; self
    }
}
