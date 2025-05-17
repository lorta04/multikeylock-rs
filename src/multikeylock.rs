use dashmap::DashMap;
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{select, spawn, time::sleep};
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

    pub async fn lock<K: Into<String>>(&self, key: K) -> Option<KeyLock> {
        self.lock_with_timeout(key, self.timeout).await
    }

    pub async fn lock_with_timeout<K: Into<String>>(
        &self,
        key: K,
        timeout: Duration,
    ) -> Option<KeyLock> {
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let handle = tokio::spawn(async move {
            sleep(timeout).await;
            cancel_clone.cancel();
        });

        let result = self.lock_with_token(key, cancel).await;
        handle.abort();

        result
    }

    pub async fn lock_with_token<K: Into<String>>(
        &self,
        key: K,
        cancel: CancellationToken,
    ) -> Option<KeyLock> {
        let key: String = key.into();
        let token_id = GLOBAL_COUNTER.fetch_add(1, Ordering::SeqCst);

        loop {
            let loaded = self.locks.entry(key.clone()).or_insert(token_id);
            if *loaded == token_id {
                return Some(KeyLock {
                    map: self.locks.clone(),
                    key,
                    token_id,
                });
            }

            select! {
                _ = cancel.cancelled() => return None,
                _ = sleep(self.retry) => {}
            }
        }
    }
}

#[derive(Debug)]
pub struct KeyLock {
    map: Arc<DashMap<String, u64>>,
    pub key: String,
    token_id: u64,
}

impl Drop for KeyLock {
    fn drop(&mut self) {
        self.map.remove_if(&self.key, |_, v| *v == self.token_id);
    }
}
