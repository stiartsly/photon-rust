
pub(crate) const MAX_ENTRIES_PER_BUCKET: usize = 8;
pub(crate) const BUCKET_REFRESH_INTERVAL: u128 = 15 * 60 * 1000;

pub(crate) const KBUCKET_OLD_AND_STALE_TIMEOUT: i32 = 2;

pub(crate) const KBUCKET_OLD_AND_STALE_TIME: u128 = 15 * 60 * 1000;
pub(crate) const KBUCKET_PING_BACKOFF_BASE_INTERVAL: u128 = 60 * 1000;

pub(crate) const KBUCKET_MAX_TIMEOUTS: i32 = 5;
