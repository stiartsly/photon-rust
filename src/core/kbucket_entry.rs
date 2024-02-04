use std::fmt;
use std::net::SocketAddr;
use std::time::SystemTime;

use crate::{
    as_millis,
    constants,
    version,
    id::Id,
    node::{Node, Reachable}
};

/**
 * Entry in a KBucket, it basically contains an IP address of a node,
 * the UDP port of the node and a node id.
 */
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
        KBucketEntry {
            node: Node::new(id, addr),
            created: SystemTime::UNIX_EPOCH,
            last_seen: SystemTime::UNIX_EPOCH,
            last_sent: SystemTime::UNIX_EPOCH,
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

    pub(crate) fn set_version(&mut self, ver: i32)  {
        self.node.set_version(ver)
    }

    pub(crate) fn ceated(&self) -> SystemTime {
        self.created
    }

    pub(crate) fn last_seen(&self) -> SystemTime {
        self.last_seen
    }

    pub(crate) fn last_sent(&self) -> SystemTime {
        self.last_sent
    }

    pub(crate) fn failed_requests(&self) -> i32 {
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

    pub(crate) const fn is_eligible_for_nodes_list(&self) -> bool {
        // 1 timeout can occasionally happen. should be fine to hand it out
        // as long as we've verified it at least once
        self.reachable && self.failed_requests < 3
    }

    pub(crate) const fn is_eligible_for_local_lookup(&self) -> bool {
        // allow implicit initial ping during lookups
        // TODO: make this work now that we don't keep unverified entries
        // in the main bucket
        (self.reachable && self.failed_requests <= 3) || self.failed_requests <= 0
    }

    /**
     * Should be called to signal that a request to this node has timed out;
     */
    pub(crate) fn signal_request_timeout(&mut self) {
        if self.failed_requests <=0 {
            self.failed_requests = 1
        } else {
            self.failed_requests += 1
        }
    }

    pub(crate) fn needs_replacement(&self) -> bool {
        (self.failed_requests > 1 && !self.reachable()) ||
            (self.failed_requests > constants::KBUCKET_MAX_TIMEOUTS && self.old_and_stale())
    }

    pub(crate) fn needs_ping(&self) -> bool {
        // don't ping if recently seen to allow NAT entries to time out
        // see https://arxiv.org/pdf/1605.05606v1.pdf for numbers
        // and do exponential backoff after failures to reduce traffic
        if as_millis!(&self.last_seen) < 30 * 1000
            || self.within_backoff_window(&self.last_seen) {
            return false;
        }

        self.failed_requests != 0 ||
            as_millis!(&self.last_seen) > constants::KBUCKET_OLD_AND_STALE_TIME
    }

    pub(crate) fn merge(&mut self, other: &Box<Self>) {
        if !self.equals(other) {
            return
        }

        self.created = self.created.max(other.created);
        self.last_seen = self.last_seen.max(other.last_seen);
        self.last_sent = self.last_sent.max(other.last_sent);

        if other.reachable() {
            self.set_reachable(true);
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
        as_millis!(&self.last_sent) < backoff
    }

    fn old_and_stale(&self) -> bool {
        self.failed_requests > constants::KBUCKET_OLD_AND_STALE_TIMEOUT &&
            as_millis!(&self.last_seen) >
                constants::KBUCKET_OLD_AND_STALE_TIME
    }

    pub(crate) fn equals(&self, other: &Self) -> bool {
        self.node == other.node
    }

    pub(crate) fn matches(&self, other: &Self) -> bool {
        self.node.matches(&other.node)
    }
}

impl Reachable for KBucketEntry {
    fn reachable(&self) -> bool {
        self.reachable
    }

    fn unreachable(&self) -> bool {
        self.last_sent == SystemTime::UNIX_EPOCH
    }

    fn set_reachable(&mut self, reachable: bool) {
        self.reachable = reachable
    }
}

impl PartialEq for KBucketEntry {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }

    fn ne(&self, other: &Self) -> bool {
        self.node != other.node
    }
}

impl fmt::Display for KBucketEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{};seen:{}; age:{}",
            self.node.id(),
            self.node.socket_addr().ip(),
            as_millis!(&self.last_seen),
            as_millis!(&self.created)
        )?;

        if self.last_sent.elapsed().is_ok() {
            write!(f, "; sent:{}", as_millis!(&self.last_sent))?;
        }
        if self.failed_requests > 0 {
            write!(f, "; fail: {}", self.failed_requests - 0)?;
        }
        if self.reachable {
            write!(f, "; reachable")?;
        }
        if self.node.version() != 0 {
            write!(f, "; ver: {}",
                version::formatted_version(self.node.version())
            )?;
        }
        Ok(())
    }
}
