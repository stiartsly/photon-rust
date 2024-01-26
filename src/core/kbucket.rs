
use std::fmt;
use std::net::SocketAddr;
use std::time::SystemTime;
use std::collections::LinkedList;

use libsodium_sys::randombytes_uniform;
use log::{info};

use crate::constants;
use crate::id::Id;
use crate::node::CheckReach;
use crate::prefix::Prefix;
use crate::kbucket_entry::KBucketEntry;

macro_rules! as_millis {
    ($time:expr) => {{
        $time.elapsed().unwrap().as_millis()
    }};
}

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

    entries: LinkedList<Box<KBucketEntry>>,
    last_refresh: SystemTime,
}

#[allow(dead_code)]
impl KBucket {
    pub(crate) fn new(prefix: &Prefix, is_home: bool) -> Self {
        KBucket {
            prefix: prefix.clone(),
            home_bucket: is_home,
            entries: LinkedList::new(),
            last_refresh: SystemTime::UNIX_EPOCH
        }
    }

    pub(crate) const fn is_home_bucket(&self) -> bool {
        self.home_bucket
    }

    pub(crate) fn entries(&self) -> &LinkedList<Box<KBucketEntry>> {
        &self.entries
    }

    pub(crate) fn size(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn is_full(&self) -> bool {
        self.entries.len() >= crate::constants::MAX_ENTRIES_PER_BUCKET
    }

    pub(crate) fn random(&self) -> Option<&Box<KBucketEntry>> {
        let len = self.entries.len();
        if len == 0 {
            return None;
        }

        let rand;
        unsafe {
            rand = randombytes_uniform(self.entries.len() as u32);
        }

        let mut iter = self.entries.iter();
        let mut index = 0;
        while index < rand {
            iter.next();
            index += 1;
        }
        iter.next()
    }

    pub(crate) fn entry(&self, id: &Id) -> Option<&Box<KBucketEntry>> {
        self.find_any(|item| item.id() == id)
    }

    pub(crate) fn find(&self, id: &Id, addr: &SocketAddr ) -> Option<&Box<KBucketEntry>> {
        self.find_any(|item| item.id() == id || item.node().socket_addr() == addr)
    }

    pub(crate) fn exist(&self, id: &Id) -> bool {
        self.find_any(|item| item.id() == id).is_some()
    }

    pub(crate) fn needs_refreshing(&self) -> bool {
        as_millis!(&self.last_refresh) > constants::BUCKET_REFRESH_INTERVAL
            && self.find_any(|item| item.needs_ping()).is_some()
    }

    pub(crate) fn needs_replacement(&self) -> bool {
        self.find_any(|item| item.needs_replacement()).is_some()
    }

    pub(crate) fn update_refresh_time(&mut self) {
        self.last_refresh = SystemTime::now()
    }

    pub(crate) fn _put(&mut self, entry: &Box<KBucketEntry>) {
        self.entries.iter_mut().for_each(|item |{
            if item == item {
                item.merge(entry);
                return;
            }

            // Node id and address conflict
            // Log the conflict and keep the existing entry
            if entry.matches(item) {
                info!("New node {} claims same ID or IP as  {}, might be impersonation attack or IP change.
                    ignoring until old entry times out", entry, item);
                return;
            }
        });

        if entry.reachable() {
            if self.entries.len() < constants::MAX_ENTRIES_PER_BUCKET {
                // insert to the list if it still has room
                // TODO: _update(nullptr, newEntry);
                return;
            }

            // Try to replace the bad entry
            if self._replace_bad_entry_with(entry) {
                return;
            }

            // TODO;
        }
    }

    fn _remove_if_bad(&mut self, to_remove: &Box<KBucketEntry>, force: bool) {
        if (force || to_remove.needs_replacement()) &&
            self.exist(to_remove.id())  {
            self._update_with_remove_or_insert(Some(&to_remove), None)
        }
    }

    fn _update(&mut self, to_refresh: &Box<KBucketEntry>) {
        for item in self.entries.iter_mut() {
            if to_refresh.eq(item) {
                item.merge(&to_refresh);
                return;
            }
        }
    }

    fn _on_timeout(&mut self, _: &Id) {
        unimplemented!();
        /* TODO:
        if let Some(item) = self.entries.iter_mut().find(|item| item.id() == id) {
            item.signal_request_timeout();
            self._remove_if_bad(item, false);
        }
        */
    }

    fn _on_send(&mut self, id: &Id) {
        self.entries.iter_mut().for_each(|item | {
            if item.id() == id {
                item.signal_request();
                return;
            }
        })
    }

    fn _replace_bad_entry_with(&mut self, new_entry: &Box<KBucketEntry>) -> bool {
        for item in self.entries.iter_mut() {
            if item.needs_replacement() {
                self._update(new_entry);
                return true;
            }
        }
        return false;
    }

    fn _update_with_remove_or_insert(&mut self,
        _: Option<&Box<KBucketEntry>>,
        _: Option<&Box<KBucketEntry>>) {

        unimplemented!()
    }

    fn find_any<P>(&self, predicate: P) -> Option<&Box<KBucketEntry>>
    where P: Fn(&KBucketEntry) -> bool {
        self.entries.iter().find(|item| predicate(&item))
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
        for item in self.entries.iter() {
            write!(f, " {}", item)?;
        }
        Ok(())
    }
}
