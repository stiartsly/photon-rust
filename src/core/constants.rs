pub(crate) const MAX_ENTRIES_PER_BUCKET: usize = 8;

// Refresh interval for a bucket in milliseconds
pub(crate) const BUCKET_REFRESH_INTERVAL: u128 = 15 * 60 * 1000;

// Maximum number of timeouts for considering a K-bucket entry as old and stale
pub(crate) const KBUCKET_OLD_AND_STALE_TIMEOUT: i32 = 2;

// Time threshold for considering a K-bucket entry as old and stale in milliseconds
pub(crate) const KBUCKET_OLD_AND_STALE_TIME: u128 = 15 * 60 * 1000;

// Base interval for backoff when sending ping messages to nodes in milliseconds
pub(crate) const KBUCKET_PING_BACKOFF_BASE_INTERVAL: u128 = 60 * 1000;

// Maximum number of timeouts before considering a K-bucket entry as unresponsive
pub(crate) const KBUCKET_MAX_TIMEOUTS: i32 = 5;

pub(crate) const RE_ANNOUNCE_INTERVAL: u64 = 5 * 60 * 1000;


pub(crate) const DHT_UPDATE_INTERVAL:u64 = 1000;
pub(crate) const RANDOM_LOOKUP_INTERVAL: u64 = 10 * 60 * 1000;  // 10 minutes
pub(crate) const RANDOM_PING_INTERVAL: u64 = 10 * 1000;         // 10 seconds
