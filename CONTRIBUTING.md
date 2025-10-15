# Contributing to scheduled-rs

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

## Getting Started

1. **Fork the repository**
2. **Clone your fork**
   ```bash
   git clone https://github.com/yourusername/scheduled-rs.git
   cd scheduled-rs
   ```
3. **Create a branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Cargo

### Building

```bash
# Build all crates
cargo build --all

# Run tests
cargo test --all

# Run examples
cargo run --example main
```

### Development Tools

Install helpful development tools:

```bash
make dev-deps
```

This installs:
- `cargo-watch` - Auto-rebuild on file changes
- `cargo-expand` - View expanded macros

## Code Style

### Formatting

Run `cargo fmt` before committing:

```bash
make fmt
```

### Linting

Run `clippy` to catch common mistakes:

```bash
make clippy
```

### All Checks

Run all checks at once:

```bash
make check-all
```

## Testing

### Unit Tests

Add tests for new features:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Your test
    }
}
```

### Integration Tests

Add integration tests in `scheduled/tests/`:

```rust
#[tokio::test]
async fn test_scheduled_task() {
    // Test scheduled tasks
}
```

## Project Structure

```
scheduled-rs/
├── scheduled/           # Main library (public API)
├── scheduled-macro/     # Procedural macros
├── scheduled-runtime/   # Runtime and scheduler logic
├── examples/           # Example applications
└── tests/             # Integration tests
```

## Pull Request Process

1. **Update documentation** - If you add features, update README.md and doc comments
2. **Add tests** - Ensure your changes are tested
3. **Run checks** - `make check-all` should pass
4. **Update CHANGELOG** - Add a note about your changes
5. **Create PR** - With clear description of changes

### PR Title Format

Use conventional commit format:

- `feat: Add new feature`
- `fix: Fix bug in scheduler`
- `docs: Update README`
- `refactor: Improve code structure`
- `test: Add tests for X`
- `chore: Update dependencies`

## Feature Requests

Open an issue with:
- Clear description of the feature
- Use cases / motivation
- Example API (if applicable)

## Bug Reports

Include:
- Rust version
- Operating system
- Minimal reproduction example
- Expected vs actual behavior
- Error messages / stack traces

## Code Review

All submissions require review. We use GitHub pull requests for this purpose.

## Areas for Contribution

- 📚 **Documentation** - Improve docs, add examples
- 🐛 **Bug Fixes** - Fix reported issues
- ✨ **Features** - Implement new scheduling features
- 🧪 **Tests** - Improve test coverage
- 🎨 **Examples** - Add real-world examples
- 🌍 **Localization** - Translate documentation

## Development Guidelines

### Adding a New Feature

1. Discuss in an issue first
2. Update macro if needed (`scheduled-macro`)
3. Update runtime if needed (`scheduled-runtime`)
4. Update public API (`scheduled`)
5. Add tests
6. Add examples
7. Update documentation

### Macro Development

When modifying `scheduled-macro/src/lib.rs`:

```bash
# View expanded macros
cd scheduled && cargo expand
```

### Runtime Development

When modifying `scheduled-runtime/src/lib.rs`, ensure:
- Backward compatibility
- Error handling
- Documentation

## Documentation

### Doc Comments

Use doc comments for public API:

```rust
/// Brief description
///
/// # Examples
///
/// ```rust
/// #[scheduled(fixed_rate = 30)]
/// async fn my_task() {
///     // ...
/// }
/// ```
///
/// # Errors
///
/// Returns error if...
pub fn my_function() {
    // ...
}
```

### README Updates

Update README.md when:
- Adding features
- Changing API
- Adding examples

## Release Process

(For maintainers)

1. Update version in all `Cargo.toml` files
2. Update CHANGELOG.md
3. Create git tag
4. Publish to crates.io:
   ```bash
   cd scheduled-macro && cargo publish
   cd ../scheduled-runtime && cargo publish
   cd ../scheduled && cargo publish
   ```

## Questions?

Feel free to open an issue for questions or join our discussions!

## License

By contributing, you agree that your contributions will be licensed under MIT OR Apache-2.0.