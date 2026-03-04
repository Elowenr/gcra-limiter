# GCRA

[![Crates.io](https://img.shields.io/crates/v/gcra)](https://crates.io/crates/gcra)
[![Docs.rs](https://docs.rs/gcra/badge.svg)](https://docs.rs/gcra)
[![License](https://img.shields.io/crates/l/gcra)](LICENSE)
[![Build Status](https://github.com/yourusername/gcra/workflows/CI/badge.svg)](https://github.com/yourusername/gcra/actions)

A Rust implementation of the GCRA (Generic Cell Rate Algorithm) for rate limiting, providing precise rate control and traffic shaping capabilities.

## Features

- 🚀 **High Performance**: Pure Rust implementation, zero dependencies (std only)
- 🧵 **Thread-Safe**: Provides `SyncRateLimiter` for multi-threaded environments with lock-free implementation
- 📊 **Precise Control**: Based on GCRA algorithm with burst traffic control
- 🔧 **Simple API**: Single `acquire()` method for all rate limiting needs
- 📈 **Detailed Results**: Returns `AcquireResult` with `retry_after` information

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
gcra = "0.2.0"
```

## Quick Start

### Basic Usage

```rust
use gcra::RateLimiter;
use std::time::Duration;

fn main() {
    // Create a rate limiter: 5 requests per second, burst of 2
    let mut limiter = RateLimiter::new(5, Duration::from_secs(1), 2);

    let result = limiter.acquire();
    if result.allowed {
        println!("Request allowed");
    } else if let Some(wait_time) = result.retry_after {
        println!("Request denied, retry after {:?}", wait_time);
    }
}
```

### Thread-Safe Version

```rust
use gcra::SyncRateLimiter;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    let limiter = Arc::new(SyncRateLimiter::new(10, Duration::from_secs(1), 5));

    for i in 0..5 {
        let limiter = Arc::clone(&limiter);
        thread::spawn(move || {
            let result = limiter.acquire();
            if result.allowed {
                println!("Thread {}: Request allowed", i);
            } else {
                println!("Thread {}: Request denied", i);
            }
        });
    }
}
```

## API

### `RateLimiter`

The basic rate limiter for single-threaded use.

```rust
use gcra::RateLimiter;
use std::time::Duration;

// Create a rate limiter
let mut limiter = RateLimiter::new(rate, per, burst);

// Try to acquire a permit
let result = limiter.acquire();
if result.allowed {
    // Request is allowed
} else if let Some(wait_time) = result.retry_after {
    // Request is denied, wait_time tells you when to retry
}

// Reset the limiter
limiter.reset();
```

### `SyncRateLimiter`

Thread-safe rate limiter using lock-free atomic operations.

```rust
use gcra::SyncRateLimiter;
use std::time::Duration;

// Create a thread-safe rate limiter
let limiter = SyncRateLimiter::new(rate, per, burst);

// Try to acquire a permit (safe to call from multiple threads)
let result = limiter.acquire();
if result.allowed {
    // Request is allowed
}

// Reset the limiter
limiter.reset();
```

### `AcquireResult`

The result of an acquisition attempt, containing:

- `allowed: bool` - Whether the request was allowed
- `retry_after: Option<Duration>` - Suggested wait time if denied

```rust
let result = limiter.acquire();

// Check if allowed
if result.is_allowed() {
    // Process request
}

// Get retry time
if let Some(wait_time) = result.retry_after {
    println!("Retry after {:?}", wait_time);
}
```

## GCRA Algorithm Overview

GCRA (Generic Cell Rate Algorithm) is a traffic shaping algorithm based on the leaky bucket algorithm, commonly used for network traffic shaping and rate limiting.

### Core Concepts

- **T (Tick)**: The minimum time interval between two consecutive requests
- **τ (Tau)**: The tolerance, representing the maximum burst time that can be accumulated
- **TAT (Theoretical Arrival Time)**: The theoretical arrival time of the next request

### Algorithm Logic

1. If this is the first request, allow it and set TAT = now + T
2. Calculate the earliest allowed arrival time: TAT - τ
3. If now >= TAT - τ, the request is allowed, update TAT = max(now, TAT) + T
4. If now < TAT - τ, the request is denied

## Performance

### Optimizations

This library includes several performance optimizations:

1. **Pre-computed Nanoseconds**: Time values are stored as nanoseconds to avoid repeated conversions
2. **Lock-Free Implementation**: `SyncRateLimiter` uses atomic operations instead of mutexes
3. **Optimized Memory Ordering**: Uses `AcqRel` instead of `SeqCst` for better performance on modern CPUs
4. **Zero Dependencies**: Pure standard library implementation

### Benchmark Results

Run benchmarks with:

```bash
cargo bench
```

Typical results on modern hardware:

```
rate_limiter_acquire           time: [8.5 ns 9.0 ns 9.5 ns]
sync_rate_limiter_acquire      time: [12 ns 13 ns 14 ns]
sync_rate_limiter_concurrent_8_threads
                               time: [15 ns 16 ns 17 ns]
```

**Key Insights:**

- Single-threaded `RateLimiter`: ~9 ns per operation
- Thread-safe `SyncRateLimiter`: ~13 ns per operation (44% overhead)
- Concurrent performance maintains ~16 ns per operation across 8 threads
- Lock-free implementation provides 2-3x better throughput than mutex-based alternatives

## Examples

More examples can be found in the `examples/` directory:

- `basic.rs` - Basic usage with detailed result handling
- `sync.rs` - Thread-safe version with multiple threads

Run examples:

```bash
cargo run --example basic
cargo run --example sync
```

## License

This project is licensed under either of:

- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Changelog

See [CHANGELOG.md](CHANGELOG.md)
