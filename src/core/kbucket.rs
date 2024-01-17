
use std::fmt;
use std::time::SystemTime;
use std::collections::LinkedList;
use libsodium_sys::randombytes_uniform;
use log::{info};

use crate::id::Id;
use crate::prefix::Prefix;
use crate::node::Visit;
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

        let rng_index;
        unsafe {
            rng_index = randombytes_uniform(self.entries.len() as u32);
        }

        let mut iter = self.entries.iter();
        let mut index = 0;
        while index < rng_index {
            iter.next();
            index += 1;
        }
        iter.next()
    }

    pub(crate) fn kentry(&self, id: &Id) -> Option<&KBucketEntry> {
        return self.find_any(|item| item.node_id() == id);
    }

    fn _on_timeout(&mut self, id: &Id) {
        self.entries.iter_mut().for_each(|item | {
            if item.node_id() == id {
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
            if item.node_id() == id {
                item.signal_request();
                return;
            }
        })
    }

    fn _put(&mut self, new_entry: &KBucketEntry) {
        for item in self.entries.iter_mut() {
            if item == item {
                item.merge_with(new_entry);
                return;
            }

            // Node id and address conflict
            // Log the conflict and keep the existing entry
            if new_entry.matches(item) {
                info!("New node {} claims same ID or IP as  {}, might be impersonation attack or IP change.
                    ignoring until old entry times out", new_entry, item);
                return;
            }
        }

        if new_entry.reachable() {
            if self.entries.len() < constants::MAX_ENTRIES_PER_BUCKET {
                // insert to the list if it still has room
                // TODO: _update(nullptr, newEntry);
                return;
            }

            // Try to replace the bad entry
            if self._replace_bad_entry(new_entry) {
                return;
            }

            // TODO;
        }
    }

    fn _replace_bad_entry(&mut self, new_entry: &KBucketEntry) -> bool {
        for item in self.entries.iter_mut() {
            if item.needs_replancement() {
                self._update(new_entry);
                return true;
            }
        }
        return false;
    }

    fn _update(&mut self, to_refresh: &KBucketEntry) {
        for item in self.entries.iter_mut() {
            if to_refresh.eq(item) {
                item.merge_with(to_refresh);
                return;
            }
        }
    }

    fn find_any<P>(&self, predicate: P) -> Option<&KBucketEntry> where P: Fn(&KBucketEntry) -> bool {
        for item in self.entries.iter() {
            if predicate(&item) {
                return Some(&item);
            }
        }
        None
    }

    fn any_match<F>(&self, predicate: F) -> bool where F: Fn(&KBucketEntry) -> bool {
        self.find_any(predicate).is_some()
    }
}

impl fmt::Display for KBucket {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO:
        Ok(())
    }
}