# Contributing to Concerto

Thank you for your interest in contributing to Concerto! This guide will help you get started.

## 🚀 Quick Start

1. **Fork and clone the repository**
   ```bash
   git clone https://github.com/yourusername/concerto.git
   cd concerto
   ```

2. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes and test**
   ```bash
   cargo test --all
   cargo fmt
   cargo clippy
   ```

4. **Commit and push**
   ```bash
   git add .
   git commit -m "feat: your feature description"
   git push origin feature/your-feature-name
   ```

5. **Create a Pull Request**

## 🏗️ Development Setup

### Prerequisites
- Rust 1.70 or later
- Cargo

### Building
```bash
# Build all workspace members
cargo build --all

# Run tests
cargo test --all

# Run examples
cargo run --example basic
cargo run --example method-scheduled
```

### Code Quality
```bash
# Format code
cargo fmt --all

# Lint code
cargo clippy --all -- -D warnings

# View expanded macros
cd concerto && cargo expand
```

## 📝 Commit Convention

Use conventional commit format:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Test additions/changes
- `chore:` - Maintenance tasks

Examples:
```
feat: add support for custom time zones
fix: resolve config placeholder parsing issue
docs: update README with new examples
refactor: simplify scheduler registration logic
```

## 🧪 Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Your test
    }
}
```

### Integration Tests
Place integration tests in `tests/` directory:
```rust
#[tokio::test]
async fn test_scheduler_lifecycle() {
    // Test scheduler
}
```

## 📚 Documentation

### Doc Comments
Use doc comments for public APIs:
```rust
/// Brief description of the function
///
/// # Examples
///
/// ```rust
/// #[scheduled(fixed_rate = "30s")]
/// async fn my_task() {
///     println!("Task running");
/// }
/// ```
///
/// # Errors
///
/// Returns error if...
pub fn my_function() {}
```

### README Updates
Update README.md when:
- Adding features
- Changing public API
- Adding examples or use cases

## 🎯 Areas for Contribution

- 📚 **Documentation** - Improve docs, add examples
- 🐛 **Bug Fixes** - Fix reported issues
- ✨ **Features** - Implement new scheduling features
- 🧪 **Tests** - Improve test coverage
- 🎨 **Examples** - Add real-world examples
- 🔍 **Code Review** - Review pull requests

## 🔧 Project Structure

```
concerto/
├── concerto/              # Main library (public API)
├── concerto-macro/        # Procedural macros
├── concerto-runtime/      # Runtime and scheduler logic
├── examples/              # Example applications
└── tests/                # Integration tests
```

## 📋 Pull Request Process

1. **Update documentation** - Add/update docs for new features
2. **Add tests** - Ensure changes are tested
3. **Run checks** - All checks must pass
4. **Update CHANGELOG** - Add entry in Unreleased section
5. **Clear description** - Explain what and why

### PR Checklist
- [ ] Code builds without errors
- [ ] All tests pass
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] CHANGELOG updated
- [ ] Examples added/updated if needed

## 🐛 Bug Reports

Include:
- Rust version
- Operating system
- Minimal reproduction example
- Expected vs actual behavior
- Error messages/stack traces

## 💡 Feature Requests

Open an issue with:
- Clear description
- Use cases / motivation
- Proposed API (if applicable)
- Example code

## 📜 License

By contributing, you agree that your contributions will be licensed under MIT OR Apache-2.0.

## ❓ Questions?

Feel free to open an issue for questions or start a discussion!

---

Thank you for contributing to Concerto! 🎵

Thank you for contributing to Concerto! �
