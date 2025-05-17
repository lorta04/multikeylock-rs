use dashmap::DashMap;
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_RETRY: Duration = Duration::from_millis(10);

static GLOBAL_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct Config {
    pub map: DashMap<String, u64>,
    pub timeout: Option<Duration>,
    pub retry: Option<Duration>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            map: DashMap::new(),
            timeout: Some(DEFAULT_TIMEOUT),
            retry: Some(DEFAULT_RETRY),
        }
    }
}

#[derive(Debug)]
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
            locks: Arc::new(config.map),
            timeout: config.timeout.unwrap_or_else(|| DEFAULT_TIMEOUT),
            retry: config.retry.unwrap_or_else(|| DEFAULT_RETRY),
        }
    }

    pub async fn lock<K: Into<String>>(&self, key: K) -> bool {
        self.lock_with_timeout(key, self.timeout).await
    }

    pub async fn lock_with_timeout<K: Into<String>>(&self, key: K, timeout: Duration) -> bool {
        let cancel = CancellationToken::new();

        tokio::spawn({
            let cancel = cancel.clone();
            async move {
                tokio::time::sleep(timeout).await;
                cancel.cancel();
            }
        });

        self.lock_with_token(key, cancel).await
    }

    pub async fn lock_with_token<K: Into<String>>(
        &self,
        key: K,
        cancel: CancellationToken,
    ) -> bool {
        let key: String = key.into();
        let token_id = GLOBAL_COUNTER.fetch_add(1, Ordering::SeqCst);

        loop {
            let loaded = self.locks.entry(key.clone()).or_insert(token_id);
            if *loaded == token_id {
                return true;
            }

            tokio::select! {
                _ = cancel.cancelled() => return false,
                _ = sleep(self.retry) => {}
            }
        }
    }
}
