use std::collections::BTreeMap;
use std::fmt;
use std::net::SocketAddr;
use std::time::SystemTime;

use libsodium_sys::randombytes_uniform;
use log::info;

use crate::{
    as_millis, constants,
    id::Id,
    prefix::Prefix,
    node_info::Reachable,
    kbucket_entry::KBucketEntry,
};

/**
 * A KBucket is just a list of KBucketEntry objects.
 *
 * The list is sorted by time last seen : The first element is the least
 * recently seen, the last the most recently seen.
 *
 * CAUTION:
 *   All methods name leading with _ means that method will WRITE the
 *   list, it can only be called inside the routing table's
 *   pipeline processing.
 *
 *   Due the heavy implementation the stream operations are significant
 *   slow than the for-loops. so we should avoid the stream operations
 *   on the KBucket entries and the cache entries, use for-loop instead.
 */
pub(crate) struct KBucket {
    prefix: Prefix,
    home_bucket: bool,

    entries: BTreeMap<Id, Box<KBucketEntry>>,
    last_refreshed: SystemTime,
}

#[allow(dead_code)]
impl KBucket {
    pub(crate) fn new(prefix: Prefix, is_home_bucket: bool) -> Self {
        Self {
            prefix,
            home_bucket: is_home_bucket,
            entries: BTreeMap::new(),
            last_refreshed: SystemTime::UNIX_EPOCH,
        }
    }

    pub(crate) fn prefix(&self) -> &Prefix {
        &self.prefix
    }

    pub(crate) fn is_home_bucket(&self) -> bool {
        self.home_bucket
    }

    pub(crate) fn entries(&self) -> &BTreeMap<Id, Box<KBucketEntry>> {
        &self.entries
    }

    pub(crate) fn size(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn is_full(&self) -> bool {
        self.entries.len() >= constants::MAX_ENTRIES_PER_BUCKET
    }

    pub(crate) fn random(&self) -> Option<&Box<KBucketEntry>> {
        let rand = unsafe {
            randombytes_uniform(self.entries.len() as u32)
        } as usize;
        self.entries.iter().nth(rand).map(|(_,v)|v)
    }

    pub(crate) fn entry(&self, id: &Id) -> Option<&Box<KBucketEntry>> {
        self.entries.get(id)
    }

    pub(crate) fn pop(&mut self) -> Option<Box<KBucketEntry>> {
        self.entries.pop_first().map(|(_,v)|v)
    }

    pub(crate) fn find(&self, id: &Id, addr: &SocketAddr) -> Option<&Box<KBucketEntry>> {
        self.find_any(|item| item.node_id() == id || item.node_addr() == addr)
    }

    pub(crate) fn exists(&self, id: &Id) -> bool {
        self.entries.contains_key(id)
    }

    pub(crate) fn needs_refreshing(&self) -> bool {
        as_millis!(&self.last_refreshed) > constants::BUCKET_REFRESH_INTERVAL
            && self.find_any(|item| item.needs_ping()).is_some()
    }

    pub(crate) fn needs_replacement(&self) -> bool {
        self.find_any(|item| item.needs_replacement()).is_some()
    }

    pub(crate) fn update_refresh_time(&mut self) {
        self.last_refreshed = SystemTime::now()
    }

    pub(crate) fn _put(&mut self, entry: Box<KBucketEntry>) {
        if let Some(item) = self.entries.get_mut(entry.node_id()) {
            if item.equals(&entry) {
                item.merge(&entry);
                return;
            }

            // NodeInfo id and address conflict
            // Log the conflict and keep the existing entry
            if item.matches(&entry) {
                info!("New node {} claims same ID or IP as  {}, might be impersonation attack or IP change.
                    ignoring until old entry times out", entry, item);
                return;
            }
        }

        if entry.reachable() {
            // insert to the list if it still has room
            if self.entries.len() < constants::MAX_ENTRIES_PER_BUCKET {
                self.entries.insert(entry.node_id().clone(), entry);
                return;
            }

            // Try to replace the bad entry
            if self._replace_bad_entry(entry) {
                return;
            }

            // TODO;
        }
    }

    pub(crate) fn on_timeout(&mut self, id: &Id) {
        if let Some(mut entry) = self.entries.remove(id) {
            entry.signal_request_timeout();

            // NOTICE: Test only - merge buckets
            // remove when the entry needs replacement
            // Product only removes the entry if it is bad
            if entry.needs_replacement() {
                _ = self.entries.remove(entry.node_id());
            } else {
                self.entries.insert(entry.node_id().clone(), entry);
            }
        }
    }

    pub(crate) fn on_send(&mut self, id: &Id) {
        if let Some(item) = self.entries.get_mut(id) {
            item.signal_request();
        }
    }

    fn _remove_bad_entry(&mut self, entry: &Box<KBucketEntry>, force: bool) {
        if force || entry.needs_replacement() {
            _ = self.entries.remove(entry.node_id());
        }
    }

    fn _replace_bad_entry(&mut self, new_entry: Box<KBucketEntry>) -> bool {
        let mut replaced = false;
        for (_,v) in self.entries.iter_mut() {
            if v.needs_replacement() {
                v.merge(&new_entry);
                replaced = true;
                break;
            }
        }
        replaced
    }

    fn find_any<P>(&self, mut predicate: P) -> Option<&Box<KBucketEntry>>
    where P: FnMut(&Box<KBucketEntry>) -> bool {
        self.entries.iter().find(|(_,v)| predicate(&v)).map(|(_,v)|v)
    }
}

impl fmt::Display for KBucket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Prefix:{}", self.prefix.id())?;
        if self.home_bucket {
            write!(f, "[Home]")?;
        }
        write!(f, "\n")?;
        if !self.entries.is_empty() {
            write!(f, " entries[{}]", self.entries.len())?;
        }
        for (_,item) in self.entries.iter() {
            write!(f, " {}", item)?;
        }
        Ok(())
    }
}
