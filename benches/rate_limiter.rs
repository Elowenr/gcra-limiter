use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gcra_limiter::{RateLimiter, SyncRateLimiter};
use std::time::Duration;

fn bench_rate_limiter_acquire(c: &mut Criterion) {
    c.bench_function("rate_limiter_acquire", |b| {
        let mut limiter = RateLimiter::new(1000, Duration::from_secs(1), 100);
        b.iter(|| {
            black_box(limiter.acquire());
        });
    });
}

fn bench_sync_rate_limiter_acquire(c: &mut Criterion) {
    c.bench_function("sync_rate_limiter_acquire", |b| {
        let limiter = SyncRateLimiter::new(1000, Duration::from_secs(1), 100);
        b.iter(|| {
            black_box(limiter.acquire());
        });
    });
}

fn bench_sync_rate_limiter_concurrent(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    c.bench_function("sync_rate_limiter_concurrent_8_threads", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            let limiter = Arc::new(SyncRateLimiter::new(100000, Duration::from_secs(1), 1000));

            let handles: Vec<_> = (0..8)
                .map(|_| {
                    let limiter = Arc::clone(&limiter);
                    thread::spawn(move || {
                        for _ in 0..(iters / 8) {
                            black_box(limiter.acquire());
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }

            start.elapsed()
        });
    });
}

criterion_group!(
    benches,
    bench_rate_limiter_acquire,
    bench_sync_rate_limiter_acquire,
    bench_sync_rate_limiter_concurrent
);
criterion_main!(benches);
