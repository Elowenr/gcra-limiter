//! Basic example of using the GCRA rate limiter.
//!
//! Run with: cargo run --example basic

use gcra_limiter::RateLimiter;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Basic GCRA Rate Limiter Example ===\n");

    // Create a rate limiter: 5 requests per second, burst of 2
    let mut limiter = RateLimiter::new(5, Duration::from_secs(1), 2);

    println!("Rate limiter created:");
    println!("  - Rate: 5 requests/second");
    println!("  - Burst: 2 requests\n");

    // Simulate requests
    println!("Sending 10 requests immediately...\n");

    for i in 1..=10 {
        let result = limiter.acquire();
        if result.allowed {
            println!("Request {}: ✓ Allowed", i);
        } else {
            if let Some(wait_time) = result.retry_after {
                println!("Request {}: ✗ Denied (retry after {:?})", i, wait_time);
            } else {
                println!("Request {}: ✗ Denied", i);
            }
        }
    }

    println!("\n--- Waiting for 200ms ---\n");
    thread::sleep(Duration::from_millis(200));

    println!("Trying another request after 200ms:");
    let result = limiter.acquire();
    if result.allowed {
        println!("Request: ✓ Allowed");
    } else {
        if let Some(wait_time) = result.retry_after {
            println!("Request: ✗ Denied (retry after {:?})", wait_time);
        } else {
            println!("Request: ✗ Denied");
        }
    }

    // Reset and try again
    println!("\n--- Resetting rate limiter ---");
    limiter.reset();

    let result = limiter.acquire();
    if result.allowed {
        println!("Request after reset: ✓ Allowed");
    } else {
        println!("Request after reset: ✗ Denied");
    }

    println!("\n=== Example completed ===");
}
