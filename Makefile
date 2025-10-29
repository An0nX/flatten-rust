# Makefile for Flatten Rust

.PHONY: help build test clean install fmt clippy bench release

# Default target
help:
	@echo "Available targets:"
	@echo "  build      - Build project"
	@echo "  test       - Run tests"
	@echo "  clean      - Clean build artifacts"
	@echo "  install    - Install locally"
	@echo "  fmt        - Format code"
	@echo "  clippy     - Run clippy lints"
	@echo "  bench      - Run benchmarks"
	@echo "  release    - Build optimized release"

# Build project
build:
	cargo build

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Install locally
install:
	cargo install --path .

# Format code
fmt:
	cargo fmt

# Run clippy lints
clippy:
	cargo clippy -- -D warnings

# Run benchmarks
bench:
	cargo bench

# Build optimized release
release:
	cargo build --release

# Run all checks
check: fmt clippy test

# Development workflow
dev: fmt clippy test build

# Release workflow
release-check: clean fmt clippy test release

# Size check
size: release
	@du -h target/release/flatten-rust

# Security audit
audit:
	cargo audit

# Documentation
docs:
	cargo doc --no-deps --open