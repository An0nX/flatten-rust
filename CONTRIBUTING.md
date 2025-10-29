# Contributing to Flatten Rust

Thank you for your interest in contributing to Flatten Rust! This document provides guidelines and information for contributors.

## 🤝 How to Contribute

### Reporting Bugs

- Use the [GitHub issue tracker](https://github.com/YOUR_USERNAME/flatten-rust/issues)
- Provide detailed information about the bug
- Include steps to reproduce
- Specify your OS, Rust version, and project size

### Suggesting Features

- Open an issue with the "enhancement" label
- Describe the feature and its use case
- Explain why it would be valuable

### Code Contributions

1. **Fork repository**
   ```bash
   git clone https://github.com/An0nX/flatten-rust.git
   ```

2. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**
   - Follow the existing code style
   - Add tests for new functionality
   - Update documentation if needed

4. **Run tests and checks**
   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

5. **Commit your changes**
   ```bash
   git commit -m "feat: add your feature description"
   ```

6. **Push to your fork**
   ```bash
   git push origin feature/your-feature-name
   ```

7. **Create a Pull Request**
   - Provide a clear description
   - Reference any related issues
   - Ensure CI passes

## 📝 Code Style

### Rust Guidelines

- Follow standard Rust conventions
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write clear, idiomatic Rust code

### Commit Messages

Use conventional commits:
- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `style:` for formatting changes
- `refactor:` for code refactoring
- `test:` for adding tests
- `chore:` for maintenance tasks

Example:
```
feat: add support for custom file filters

This change allows users to specify custom file filtering rules
through a new --custom-filter option.

Closes #123
```

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

### Writing Tests

- Unit tests go in the same module
- Integration tests go in `tests/`
- Add tests for new functionality
- Ensure tests cover edge cases

### Test Coverage

- Aim for high test coverage
- Test error conditions
- Test performance-critical code paths

## 📚 Documentation

### Code Documentation

- Document public APIs with `///`
- Use `rustdoc` format
- Include examples where helpful
- Explain complex algorithms

### README Updates

- Update README for user-facing changes
- Add new options to usage examples
- Keep version information current

## 🏗️ Development Setup

### Prerequisites

- Rust 1.70+
- Git
- Make (optional, for using Makefile)

### Development Commands

```bash
# Install development dependencies
cargo build

# Run development build
cargo run -- --folders ./src --output dev.md

# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check documentation
cargo doc --no-deps --open
```

### Using Makefile

```bash
# Build and test
make

# Format code
make fmt

# Lint code
make clippy

# Run tests
make test

# Build release
make build-release
```

## 🔍 Code Review Process

### Review Guidelines

- Review code for correctness
- Check for performance implications
- Verify test coverage
- Ensure documentation is updated
- Check for security considerations

### Reviewer Responsibilities

- Provide constructive feedback
- Suggest improvements
- Verify CI passes
- Ensure code follows guidelines

### Author Responsibilities

- Address reviewer feedback
- Update tests and documentation
- Ensure CI passes
- Respond to comments promptly

## 🚀 Release Process

Releases are automated through GitHub Actions:

1. Create a release tag: `git tag v0.2.0`
2. Push the tag: `git push origin v0.2.0`
3. GitHub Actions will:
   - Build binaries for all platforms
   - Create a GitHub release
   - Publish to crates.io (if configured)

## 🏆 Recognition

Contributors are recognized in:
- README.md contributors section
- Release notes
- Git commit history

## 📞 Getting Help

- **Security**: Use GitHub Security Advisory
- **General Issues**: [GitHub Issues](https://github.com/An0nX/flatten-rust/issues)
- **Security Policy**: [SECURITY.md](https://github.com/An0nX/flatten-rust/blob/main/SECURITY.md)

## 📄 License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Flatten Rust! 🎉