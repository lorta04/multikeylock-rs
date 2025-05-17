# multikeylock-rs

A lightweight multi-key mutex library in Rust using DashMap and tokio. This project is a Rust implementation of the same concept explored in the [Go version](https://github.com/lorta04/multikeylock-go) but with a greater focus on idiomatic structure, async support, and learning.

## ✨ About

This repository demonstrates how to build a per-key lock system useful when concurrent operations need to be isolated by logical keys (such as user IDs). It uses a single DashMap to track lock tokens, avoiding the need for a full Mutex per key.

The design is inspired by a [vibe-coded Go implementation](https://github.com/lorta04/multikeylock-go) but this Rust version was crafted with more care and a deeper dive into concurrency patterns, async handling, and ergonomics.

While not built for production use yet this library may be useful for educational purposes or for use in personal backend projects where scoped locking per key is needed.

## 🧱 Features

-   Per-key mutex behavior using DashMap
    
-   Fully async lock acquisition using tokio
    
-   Optional timeouts and cancellation via CancellationToken
    
-   Simple KeyLock RAII guard for safe unlocking on drop
    
-   Exponential backoff retry logic for contention scenarios
    

## 🔧 Dependencies

-   `dashmap` for fast concurrent access
    
-   `tokio` async runtime
    
-   `tokio-util` for CancellationToken
    

## 📦 Example Usage

```rust
use multikeylock::MultiKeyLock;

#[tokio::main]
async fn main() {
    let lock_map = MultiKeyLock::new();

    if let Some(_guard) = lock_map.lock("user:123").await {
        // The key "user:123" is now locked
        // Do something exclusive here
    }
    // Lock is released automatically when _guard is dropped
} 
```

## 📜 License

MIT