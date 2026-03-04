# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2024-XX-XX

### Added

- `AcquireResult` struct with detailed acquisition information
  - `allowed: bool` - Whether the request was allowed
  - `retry_after: Option<Duration>` - Suggested wait time if denied
  - Helper methods: `is_allowed()`, `is_denied()`
- Pre-computed nanosecond storage for better performance
- Optimized memory ordering using `AcqRel` instead of `SeqCst`

### Changed

- **Breaking**: `acquire()` now returns `AcquireResult` instead of `bool`
- `RateLimiter` and `SyncRateLimiter` now store time values as nanoseconds internally
- Improved performance by eliminating repeated Duration conversions
- Enhanced documentation with performance benchmarks and optimization details

### Removed

- Direct boolean return from `acquire()` method (now wrapped in `AcquireResult`)

## [0.1.0] - 2024-XX-XX

### Added

- Initial release of the GCRA library
- `RateLimiter` - Basic GCRA rate limiter implementation
  - `new(rate, per, burst)` - Create a rate limiter with specified rate and burst
  - `acquire()` - Try to acquire a permit, returns `true` if allowed, `false` otherwise
  - `reset()` - Reset the rate limiter state
- `SyncRateLimiter` - Thread-safe rate limiter using lock-free atomic operations
  - Same API as `RateLimiter` but safe for concurrent use
  - Uses `AtomicU64` and CAS operations for lock-free updates

[Unreleased]: https://github.com/yourusername/gcra/compare/v0.1.0...HEAD
[0.2.0]: https://github.com/yourusername/gcra/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/yourusername/gcra/releases/tag/v0.1.0
