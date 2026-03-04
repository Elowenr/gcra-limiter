# Contributing to GCRA

Thank you for your interest in contributing to GCRA! We welcome contributions from everyone.

## Code of Conduct

This project and everyone participating in it is governed by our commitment to provide a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues to see if the problem has already been reported. When you are creating a bug report, please include as many details as possible:

- **Use a clear and descriptive title**
- **Describe the exact steps to reproduce the problem**
- **Provide specific examples to demonstrate the steps**
- **Describe the behavior you observed and what behavior you expected**
- **Include code samples and stack traces if applicable**

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

- **Use a clear and descriptive title**
- **Provide a step-by-step description of the suggested enhancement**
- **Provide specific examples to demonstrate the enhancement**
- **Explain why this enhancement would be useful**

### Pull Requests

1. Fork the repository
2. Create a new branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run the tests (`cargo test`)
5. Run clippy (`cargo clippy`)
6. Format your code (`cargo fmt`)
7. Commit your changes (`git commit -m 'Add amazing feature'`)
8. Push to the branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Cargo

### Building

```bash
git clone https://github.com/yourusername/gcra.git
cd gcra
cargo build
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run doc tests
cargo test --doc

# Run examples
cargo run --example basic
cargo run --example sync
cargo run --example weighted
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Generate documentation
cargo doc --no-deps

# Check for common mistakes
cargo check
```

## Style Guidelines

### Rust Code Style

- Follow the [Rust Style Guide](https://doc.rust-lang.org/style/)
- Use `cargo fmt` to format your code
- Use `cargo clippy` to catch common mistakes
- Write documentation for all public items
- Add examples to documentation where appropriate

### Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests liberally after the first line

Example:
```
Add support for async rate limiting

- Implement AsyncRateLimiter using tokio
- Add tests for async functionality
- Update documentation

Fixes #123
```

### Documentation

- Use `///` for doc comments
- Include examples in doc comments
- Document panics, errors, and safety considerations
- Keep documentation up to date with code changes

Example:
```rust
/// Creates a new rate limiter.
///
/// # Arguments
///
/// * `rate` - The number of requests allowed per time period
/// * `per` - The time period
/// * `burst` - The maximum burst size
///
/// # Examples
///
/// ```
/// use gcra::RateLimiter;
/// use std::time::Duration;
///
/// let limiter = RateLimiter::new(10, Duration::from_secs(1), 5);
/// ```
///
/// # Panics
///
/// Panics if `rate` is 0.
pub fn new(rate: u64, per: Duration, burst: u64) -> Self {
    // ...
}
```

## Testing Guidelines

- Write unit tests for new functionality
- Add integration tests for complex features
- Ensure all tests pass before submitting PR
- Aim for high test coverage
- Test edge cases and error conditions

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create a git tag (`git tag -a v0.1.0 -m "Release version 0.1.0"`)
4. Push the tag (`git push origin v0.1.0`)
5. GitHub Actions will automatically publish to crates.io

## Questions?

Feel free to open an issue for questions or join the discussion.

Thank you for contributing! 🎉
