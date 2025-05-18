use multikeylock::multikeylock::{Config, MultiKeyLock};
use std::time::Duration;
use tokio::time::{sleep, Instant};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_lock() {
    let lock = MultiKeyLock::with_config(Config {
        timeout: Some(Duration::from_millis(100)),
        retry: Some(Duration::from_millis(10)),
        map: Default::default(),
    });

    let key = "test-key";

    // --------------------------------------------------------------------------------------------\

    let guard1 = lock.lock(key).await;
    assert!(guard1.is_some(), "Expected to acquire initial lock");

    // --------------------------------------------------------------------------------------------

    let start = Instant::now();
    let guard2 = lock.lock(key).await;
    let elapsed = start.elapsed();

    assert!(guard2.is_none(), "Expected second lock to timeout");
    assert!(
        elapsed >= Duration::from_millis(100),
        "Timeout occurred too early: {elapsed:?}"
    );

    // --------------------------------------------------------------------------------------------

    drop(guard1);

    // --------------------------------------------------------------------------------------------

    let guard3 = lock.lock_with_timeout(key, Duration::from_secs(1)).await;
    assert!(guard3.is_some(), "Expected to acquire after release");
}

#[tokio::test]
async fn test_lock_with_timeout() {
    let lock = MultiKeyLock::new();
    let key = "test-key";

    // --------------------------------------------------------------------------------------------

    let guard1 = lock.lock_with_timeout(key, Duration::from_secs(1)).await;
    assert!(guard1.is_some(), "Expected to acquire initial lock");

    // --------------------------------------------------------------------------------------------

    let start = tokio::time::Instant::now();
    let guard2 = lock
        .lock_with_timeout(key, Duration::from_millis(100))
        .await;
    let elapsed = start.elapsed();

    assert!(guard2.is_none(), "Expected timeout and not acquire lock");
    assert!(
        elapsed >= Duration::from_millis(100),
        "Timeout returned too early"
    );

    // --------------------------------------------------------------------------------------------

    drop(guard1);

    // --------------------------------------------------------------------------------------------

    let guard3 = lock.lock_with_timeout(key, Duration::from_secs(1)).await;
    assert!(guard3.is_some(), "Expected to acquire after release");
}

#[tokio::test]
async fn test_lock_with_token() {
    let lock = MultiKeyLock::new();
    let key = "test-key";

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    // --------------------------------------------------------------------------------------------

    let guard1 = lock.lock_with_token(key, CancellationToken::new()).await;
    assert!(guard1.is_some(), "Expected to acquire lock");

    // --------------------------------------------------------------------------------------------

    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        cancel_clone.cancel();
    });

    // --------------------------------------------------------------------------------------------

    let start = tokio::time::Instant::now();
    let guard2 = lock.lock_with_token(key, cancel).await;
    let elapsed = start.elapsed();

    assert!(guard2.is_none(), "Expected to cancel and not acquire lock");
    assert!(
        elapsed >= Duration::from_millis(50),
        "Cancellation occurred too quickly"
    );

    // --------------------------------------------------------------------------------------------

    drop(guard1);

    // --------------------------------------------------------------------------------------------

    let guard3 = lock.lock_with_token(key, CancellationToken::new()).await;
    assert!(guard3.is_some(), "Expected to acquire after release");
}

#[tokio::test]
async fn test_lock_precise_retry_steps() {
    let lock = MultiKeyLock::with_config(Config {
        timeout: Some(Duration::from_secs(5)),
        retry: Some(Duration::from_millis(10)),
        map: Default::default(),
    });

    let key = "test-key";

    // --------------------------------------------------------------------------------------------

    let guard1 = lock.lock(key).await;
    assert!(guard1.is_some());

    // --------------------------------------------------------------------------------------------

    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        drop(guard1);
    });

    // Second lock attempts to acquire with exponential backoff:
    //   - Try 0: immediately → fail → retry = 10ms → sleep until 10ms
    //   - Try 1:     at 10ms → fail → retry = 20ms → sleep until 30ms
    //   - Try 2:     at 30ms → fail → retry = 40ms → sleep until 70ms
    //   - Try 3:     at 70ms → fail → retry = 80ms → sleep until 150ms
    //
    // The lock should be acquired at ~150ms.
    // The next retry (if needed) would sleep until 310ms (retry = 160ms).
    // Thus, we assert the elapsed time falls between 150ms and 310ms.
    let test_start = Instant::now();
    let handle = tokio::spawn(async move {
        let guard = lock.lock(key).await;
        let elapsed = test_start.elapsed();
        (guard, elapsed)
    });

    // --------------------------------------------------------------------------------------------

    let (guard2, elapsed) = handle.await.unwrap();
    assert!(guard2.is_some(), "Expected reacquire after retry");

    // --------------------------------------------------------------------------------------------

    assert!(
        elapsed >= Duration::from_millis(150) && elapsed <= Duration::from_millis(310),
        "Expected reacquire time between 150ms and 310ms, got {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_try_lock_now() {
    let lock = MultiKeyLock::new();
    let key = "test-key";

    // --------------------------------------------------------------------------------------------

    let guard1 = lock.try_lock_now(key);
    assert!(guard1.is_some(), "Expected to acquire lock");

    // --------------------------------------------------------------------------------------------

    let guard2 = lock.try_lock_now(key);
    assert!(guard2.is_none(), "Expected to not acquire lock");

    // --------------------------------------------------------------------------------------------

    drop(guard1);

    // --------------------------------------------------------------------------------------------

    let guard3 = lock.try_lock_now(key);
    assert!(guard3.is_some(), "Expected to acquire after release");
}
