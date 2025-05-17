use dashmap::DashMap;
use std::{sync::Arc, time::Duration};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_RETRY: Duration = Duration::from_millis(10);

pub struct Config {
    pub map: Option<Arc<DashMap<String, u64>>>,
    pub timeout: Option<Duration>,
    pub retry: Option<Duration>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timeout: Some(DEFAULT_TIMEOUT),
            retry: Some(DEFAULT_RETRY),
            map: None,
        }
    }
}

pub struct MultiKeyLock {
    locks: Arc<DashMap<String, u64>>,
    pub timeout: Duration,
    pub retry: Duration,
}

impl MultiKeyLock {
    pub fn new() -> Self {
        Self::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Self {
        MultiKeyLock {
            locks: config.map.unwrap_or_else(|| Arc::new(DashMap::new())),
            timeout: config.timeout.unwrap_or_else(|| DEFAULT_TIMEOUT),
            retry: config.retry.unwrap_or_else(|| DEFAULT_RETRY),
        }
    }
}
