use std::fmt;
use std::net::{SocketAddr};
use std::time::{SystemTime};
use crate::id::Id;
use crate::nodeinfo::{NodeInfo, NodeInfoTrait};
use crate::nodeinfo::{Accessibility};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct KBucketEntry {
    nodei: NodeInfo,

    created: SystemTime,
    last_seen: SystemTime,
    last_sent: SystemTime,

    reachable: bool,
    failed_requests: i32
}

#[allow(dead_code)]
impl KBucketEntry {
    pub(crate) fn new(id: &Id, addr: &SocketAddr) -> Self {
        let unix_epoch = SystemTime::UNIX_EPOCH;

        KBucketEntry {
            nodei: NodeInfo::new(id, addr),
            created: unix_epoch,
            last_seen: unix_epoch,
            last_sent: unix_epoch,
            reachable: false,
            failed_requests: 0
        }
    }

    pub(crate) fn node_id(&self) -> &Id {
        &self.nodei.id()
    }

    pub(crate) fn node_info(&self) -> &NodeInfo {
        &self.nodei
    }

    pub(crate) fn set_version(&mut self, ver: i32) {
        self.nodei.set_version(ver);
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
        unimplemented!()
    }

    pub(crate) fn needs_ping(&self) -> bool {
        unimplemented!()
    }

    pub(crate) fn merge_with(&mut self, other: &Self) {
        if self != other {
            return;
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

    pub(crate) fn matches(&self, _: &Self) -> bool {
        unimplemented!()
    }
}

impl Accessibility for KBucketEntry {
    fn reachable(&self) -> bool {
        self.reachable
    }

    fn set_reachable(&mut self, reachable: bool) {
        self.reachable = reachable;
    }

    fn unreachable(&self) -> bool {
        self.last_sent == SystemTime::UNIX_EPOCH
    }
}

impl PartialEq for KBucketEntry {
    fn eq(&self, other: &Self) -> bool {
        self.nodei == other.nodei
    }
}

impl fmt::Display for KBucketEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@", self.nodei.id())?;
        write!(f, "{};seen:", self.nodei.socket_addr().ip())?;
        // write!(f, "{};age:", self.last_seen.into()?);
        //write!(f, "{}", (self.created - 0).to_string())?;

        //if self.lastsent > 0 {
        //    write!(f, "; sent: {}", (self.lastseen - 0).to_string())?;
        //}
        if self.failed_requests > 0 {
            write!(f, "; fail: {}", (self.failed_requests - 0).to_string())?;
        }
        if self.reachable {
            write!(f, "; reachable")?;
        }

        //version.
        Ok(())
    }
}
