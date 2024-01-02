use std::fmt;
use std::net::{SocketAddr};
use crate::id::Id;
use crate::nodeinfo::{NodeInfo, NodeInfoTrait};
use crate::nodeinfo;

#[allow(dead_code)]
pub(crate) struct KBucketEntry {
    nodeinfo: NodeInfo,

    created: u64,
    lastseen: u64,
    lastsent: u64,

    reachable: bool,
    failed_requests: i32
}

#[allow(dead_code)]
impl KBucketEntry {
    pub(crate) fn new(id: &Id, addr: &SocketAddr) -> Self {
        KBucketEntry {
            nodeinfo: NodeInfo::new(id, addr),
            created: 0,
            lastseen: 0,
            lastsent: 0,
            reachable: false,
            failed_requests: 0
        }
    }

    pub(crate) fn with_node_info(node_info: &NodeInfo) -> Self {
        KBucketEntry::new(node_info.id(), node_info.socket_addr())
    }

    pub(crate) const fn ceated(&self) -> u64 {
        self.created
    }

    pub(crate) const fn last_seen(&self) -> u64 {
        self.lastseen
    }

    pub(crate) const fn last_sent(&self) -> u64 {
        self.lastsent
    }

    pub(crate) const fn is_reachable(&self) -> bool {
        self.reachable
    }

    pub(crate) const fn failed_requests(&self) -> i32 {
        self.failed_requests
    }

    fn merge(_: &KBucketEntry) {
        unimplemented!()
    }

    pub(crate) fn signal_response(&mut self) {
        unimplemented!();
    }

    pub(crate) fn signal_request(&mut self) {
        unimplemented!();
    }

    pub(crate) fn signal_request_timeout(&mut self) {
        match self.failed_requests <=0 {
            true => self.failed_requests = 1,
            false => self.failed_requests += 1
        }
    }
}

impl nodeinfo::NodeInfoTrait for KBucketEntry {
    fn id(&self) -> &Id {
        &self.nodeinfo.id()
    }

    fn socket_addr(&self) -> &SocketAddr {
        &self.nodeinfo.socket_addr()
    }

    fn version(&self) -> i32 {
        self.nodeinfo.version()
    }

    fn set_version(&mut self, version: i32) {
        self.nodeinfo.set_version(version);
    }

    fn is_ipv4(&self) -> bool {
        self.nodeinfo.is_ipv4()
    }

    fn is_ipv6(&self) -> bool {
        self.nodeinfo.is_ipv6()
    }
}

impl fmt::Display for KBucketEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@", self.nodeinfo.id())?;
        write!(f, "{};seen:", self.nodeinfo.socket_addr().ip())?;
        write!(f, "{};age:", (self.lastseen - 0).to_string())?;
        write!(f, "{}", (self.created - 0).to_string())?;

        if self.lastsent > 0 {
            write!(f, "; sent: {}", (self.lastseen - 0).to_string())?;
        }
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
