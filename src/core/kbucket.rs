
use std::fmt;
use std::net::SocketAddr;
use std::time::SystemTime;
use std::collections::LinkedList;
use libsodium_sys::randombytes_uniform;
use log::{info};

use crate::id::Id;
use crate::prefix::Prefix;
use crate::node::CheckReach;
use crate::kbucket_entry::{KBucketEntry};
use crate::constants;

#[allow(dead_code)]
pub(crate) struct KBucket {
    prefix: Prefix,
    home_bucket:bool,

    entries: LinkedList<Box<KBucketEntry>>,
    last_refresh: SystemTime,
}

#[allow(dead_code)]
impl KBucket {
    pub(crate) fn new(prefix: &Prefix, is_home: bool) -> Self {
        KBucket {
            prefix: *prefix,
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

    pub(crate) fn needs_to_be_refreshed(&self) -> bool {
        self.last_refresh.elapsed().unwrap().as_millis() > constants::BUCKET_REFRESH_INTERVAL
            && self.find_any(|item| item.needs_ping()).is_some()
    }

    pub(crate) fn needs_replancement(&self) -> bool {
        self.find_any(|item| item.needs_replancement()).is_some()
    }

    pub(crate) fn update_refresh_time(&mut self) {
        self.last_refresh = SystemTime::now()
    }

    pub(crate) fn _put(&mut self, entry: Box<KBucketEntry>) {
        self.entries.iter_mut().for_each(|item |{
            if item == item {
                item.merge(&entry);
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
            if self._replace_bad_entry(entry) {
                return;
            }

            // TODO;
        }
    }

    pub(crate) fn _remove_if_bad(&mut self, to_remove: Box<KBucketEntry>, force: bool) {
        if (force || to_remove.needs_replancement()) &&
            self.find_any(|item | item.id() == to_remove.id()).is_some() {
            self._remove_and_insert(Some(to_remove), None)
        }
    }

    fn _on_timeout(&mut self, id: &Id) {
        self.entries.iter_mut().for_each(|item | {
            if item.id() == id {
                item.signal_request_timeout();
                // NOTICE: Product
                //   only removes the entry if it is bad
                //_removeIfBad(entry, false);
                return;
            }
        })
    }

    fn _on_send(&mut self, id: &Id) {
        self.entries.iter_mut().for_each(|item | {
            if item.id() == id {
                item.signal_request();
                return;
            }
        })
    }

    fn _replace_bad_entry(&mut self, new_entry: Box<KBucketEntry>) -> bool {
        for item in self.entries.iter_mut() {
            if item.needs_replancement() {
                self._update(new_entry);
                return true;
            }
        }
        return false;
    }

    fn _update(&mut self, to_refresh: Box<KBucketEntry>) {
        for item in self.entries.iter_mut() {
            if to_refresh.eq(item) {
                item.merge(&to_refresh);
                return;
            }
        }
    }

    fn _remove_and_insert(&mut self, _: Option<Box<KBucketEntry>>, _: Option<Box<KBucketEntry>>) {
        unimplemented!()
    }

    fn find_any<P>(&self, predicate: P) -> Option<&Box<KBucketEntry>>
    where P: Fn(&KBucketEntry) -> bool {
        for item in self.entries.iter() {
            if predicate(&item) {
                return Some(&item);
            }
        }
        None
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
