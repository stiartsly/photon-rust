
use std::fmt;
use std::collections::LinkedList;
use libsodium_sys::randombytes_uniform;

use crate::id::Id;
use crate::prefix::Prefix;
use crate::kentry::KBucketEntry;
use crate::nodeinfo::NodeInfoTrait;

#[allow(dead_code)]
pub(crate) struct KBucket {
    prefix: Prefix,
    home_bucket:bool,

    entries: LinkedList<KBucketEntry>,
    last_refresh: u64,

    // Logger
}

#[allow(dead_code)]
impl KBucket {
    pub(crate) fn new(prefix: &Prefix, is_home: bool) -> Self {
        KBucket {
            prefix: *prefix,
            home_bucket: is_home,
            entries: LinkedList::new(),
            last_refresh: 0
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

    pub(crate) fn random(&self) -> Option<&KBucketEntry> {
        if self.entries.is_empty() {
            return None;
        }

        let random_index;
        unsafe {
            random_index = randombytes_uniform(self.entries.len() as u32);
        }

        let mut iter = self.entries.iter();
        while random_index > 0 {
            iter.next();
        }
        iter.next()
    }

    fn _on_timeout(&mut self, id: &Id) {
        for entry in self.entries.iter_mut() {
            if entry.id() == id {
                entry.signal_request_timeout();
                // NOTICE: Product
                //   only removes the entry if it is bad
                //_removeIfBad(entry, false);
                return;
            }
        }
    }

    fn _on_send(&mut self, id: &Id) {
        for entry in self.entries.iter_mut() {
            if entry.id() == id {
                entry.signal_request();
                return;
            }
        }
    }
}

impl fmt::Display for KBucket {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO:
        Ok(())
    }
}