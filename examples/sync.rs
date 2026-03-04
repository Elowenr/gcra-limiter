//! Thread-safe rate limiter example.
//!
//! Run with: cargo run --example sync

use gcra_limiter::SyncRateLimiter;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Thread-Safe Rate Limiter Example ===\n");

    // Create a thread-safe rate limiter
    let limiter = Arc::new(SyncRateLimiter::new(10, Duration::from_secs(1), 3));

    println!("Rate limiter created:");
    println!("  - Rate: 10 requests/second");
    println!("  - Burst: 3 requests");
    println!("  - Threads: 5\n");

    let mut handles = vec![];

    // Spawn 5 threads, each making requests
    for thread_id in 0..5 {
        let limiter = Arc::clone(&limiter);
        let handle = thread::spawn(move || {
            let mut allowed = 0;
            let mut denied = 0;

            for request_id in 0..5 {
                let result = limiter.acquire();
                if result.allowed {
                    println!("Thread {} Request {}: ✓ Allowed", thread_id, request_id);
                    allowed += 1;
                } else {
                    println!("Thread {} Request {}: ✗ Denied", thread_id, request_id);
                    denied += 1;
                }

                // Small delay between requests
                thread::sleep(Duration::from_millis(10));
            }

            (allowed, denied)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    let mut total_allowed = 0;
    let mut total_denied = 0;

    for handle in handles {
        let (allowed, denied) = handle.join().unwrap();
        total_allowed += allowed;
        total_denied += denied;
    }

    println!("\n=== Summary ===");
    println!("Total requests: {}", total_allowed + total_denied);
    println!("Allowed: {}", total_allowed);
    println!("Denied: {}", total_denied);
    println!("\nNote: Due to thread scheduling, results may vary between runs.");
}
