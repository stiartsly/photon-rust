use std::fmt;
use std::net::{SocketAddr};
use std::time::{SystemTime, Duration};
use crate::id::Id;
use crate::node::{Node, NodeInfo, Visit};
use crate::constants;
use crate::version;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct KBucketEntry {
    node: Node,

    created: SystemTime,
    last_seen: SystemTime,
    last_sent: SystemTime,

    reachable: bool,
    failed_requests: i32
}

#[allow(dead_code)]
impl KBucketEntry {
    pub(crate) fn new(id: &Id, addr: &SocketAddr) -> Self {
        let epoch = SystemTime::UNIX_EPOCH;

        KBucketEntry {
            node: Node::new(id, addr),
            created: epoch,
            last_seen: epoch,
            last_sent: epoch,
            reachable: false,
            failed_requests: 0
        }
    }

    pub(crate) fn id(&self) -> &Id {
        &self.node.id()
    }

    pub(crate) fn node(&self) -> &Node {
        &self.node
    }

    pub(crate) fn with_version(&mut self, ver: i32) -> &Self {
        self.node.with_version(ver); self
    }

    pub(crate) const fn ceated(&self) -> SystemTime {
        self.created
    }

    pub(crate) const fn last_seen(&self) -> SystemTime {
        self.last_seen
    }

    pub(crate) const fn last_sent(&self) -> SystemTime {
        self.last_sent
    }

    pub(crate) const fn failed_requests(&self) -> i32 {
        self.failed_requests
    }

    pub(crate) fn signal_response(&mut self) {
        self.last_seen = SystemTime::now();
        self.failed_requests = 0;
        self.reachable = true;
    }

    pub(crate) fn signal_request(&mut self) {
        self.last_sent = SystemTime::now();
    }

    pub(crate) fn is_eligible_for_nodes_list(&self) -> bool {
        // 1 timeout can occasionally happen. should be fine to hand it out as long as
        // we've verified it at least once
        self.reachable && self.failed_requests < 3
    }

    /**
     * Should be called to signal that a request to this node has timed out;
     */
    pub(crate) fn signal_request_timeout(&mut self) {
        match self.failed_requests <=0 {
            true => self.failed_requests = 1,
            false => self.failed_requests += 1
        }
    }

    pub(crate) fn needs_replancement(&self) -> bool {
        (self.failed_requests > 1 && !self.reachable()) ||
            (self.failed_requests > constants::KBUCKET_MAX_TIMEOUTS && self.old_and_stale())
    }

    pub(crate) fn needs_ping(&self) -> bool {

        // don't ping if recently seen to allow NAT entries to time out
        // see https://arxiv.org/pdf/1605.05606v1.pdf for numbers
        // and do exponential backoff after failures to reduce traffic
        if self.last_seen.elapsed().unwrap() < Duration::from_millis(30 * 1000)
            || self.within_backoff_window(&self.last_seen) {
            return false;
        }

        self.failed_requests != 0 ||
            self.last_seen.elapsed().unwrap().as_millis()
                > constants::KBUCKET_OLD_AND_STALE_TIME
    }

    pub(crate) fn merge(&mut self, other: &Box<Self>) {
        if !self.equals(other) {
            return
        }

        self.created = self.created.max(other.created);
        self.last_seen = self.last_seen.max(other.last_seen);
        self.last_sent = self.last_sent.max(other.last_sent);

        if other.reachable() {
            self.with_reachable(true);
        }
        if other.failed_requests() > 0 {
            self.failed_requests = self.failed_requests.min(other.failed_requests);
        }
    }

    fn within_backoff_window(&self, _: &SystemTime) -> bool {
        let backoff = constants::KBUCKET_PING_BACKOFF_BASE_INTERVAL <<
            std::cmp::max(
                constants::KBUCKET_MAX_TIMEOUTS,
                std::cmp::min(0, self.failed_requests -1)
            );
        self.failed_requests != 0 &&
            self.last_sent.elapsed().unwrap().as_millis() < backoff
    }

    fn old_and_stale(&self) -> bool {
        self.failed_requests > constants::KBUCKET_OLD_AND_STALE_TIMEOUT &&
            self.last_seen.elapsed().unwrap().as_millis() >
                constants::KBUCKET_OLD_AND_STALE_TIME
    }

    pub(crate) fn equals(&self, other: &Self) -> bool {
        self.node == other.node
    }

    pub(crate) fn matches(&self, other: &Self) -> bool {
        self.node.matches(&other.node)
    }
}

impl Visit for KBucketEntry {
    fn reachable(&self) -> bool {
        self.reachable
    }

    fn unreachable(&self) -> bool {
        self.last_sent == SystemTime::UNIX_EPOCH
    }

    fn with_reachable(&mut self, reachable: bool) -> &mut Self {
        self.reachable = reachable; self
    }
}

impl PartialEq for KBucketEntry {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl fmt::Display for KBucketEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{};seen:{}; age:{}",
            self.node.id(),
            self.node.socket_addr().ip(),
            self.last_seen.elapsed().unwrap().as_millis(),
            self.created.elapsed().unwrap().as_millis()
        )?;

        if self.last_sent.elapsed().is_ok() {
            write!(f, "; sent:{}", self.last_sent.elapsed().unwrap().as_millis())?;
        }
        if self.failed_requests > 0 {
            write!(f, "; fail: {}", (self.failed_requests - 0).to_string())?;
        }
        if self.reachable {
            write!(f, "; reachable")?;
        }
        if self.node.version() != 0 {
            write!(f, "; ver: {}", version::readable_version(self.node.version()))?;
        }
        Ok(())
    }
}
