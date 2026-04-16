# Contributing to zvec-rust

Thank you for your interest in contributing to zvec-rust! This document provides guidelines and instructions for contributing.

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

- **Rust** (version 1.70 or later) - [Install Rust](https://rustup.rs/)
- **CMake** (for building the zvec C library)
- **C++17 compiler** (GCC, Clang, or MSVC)
- **Git**

### Development Environment Setup

1. **Clone the repository:**

   ```bash
   git clone https://github.com/sunhailin-Leo/zvec-rust.git
   cd zvec-rust
   ```

2. **Initialize the zvec submodule:**

   ```bash
   make submodule-init
   ```

3. **Build the zvec C library:**

   ```bash
   make zvec-build
   ```

4. **Install development tools:**

   ```bash
   make setup
   ```

   This installs `rustfmt`, `clippy`, `cargo-llvm-cov`, and `cargo-fuzz`.

Alternatively, you can set environment variables manually:

```bash
export ZVEC_LIB_DIR=/path/to/zvec/build/lib
export ZVEC_INCLUDE_DIR=/path/to/zvec/src/include
```

## Code Style

We follow standard Rust conventions and use automated tools to maintain code quality.

### Formatting

All code must be formatted using `rustfmt`:

```bash
make fmt
```

Check formatting without modifying files:

```bash
make fmt-check
```

### Linting

Run clippy to catch common mistakes and improve code quality:

```bash
make clippy
```

All warnings are treated as errors in CI (`-D warnings`).

### Combined Linting

Run both formatting checks and clippy:

```bash
make lint
```

## Testing

### Unit Tests

Run unit tests (no C library required):

```bash
make test-unit
# or
cargo test --workspace --lib
```

### Integration Tests

Run integration tests (requires zvec C library):

```bash
make test-integration
# or
cargo test --test integration_test
```

### All Tests

Run all tests including documentation tests:

```bash
make test-all
# or
cargo test --workspace
```

### Fuzz Testing

Fuzz testing requires the nightly toolchain:

```bash
cargo install cargo-fuzz
make fuzz
```

Individual fuzz targets:

```bash
make fuzz-types
make fuzz-schema
make fuzz-doc
make fuzz-query
```

### Code Coverage

Generate coverage reports:

```bash
cargo install cargo-llvm-cov
make coverage-html    # HTML report
make coverage-lcov    # LCOV format
make coverage-text    # Text summary
```

## Pull Request Process

1. **Fork the repository** and create your feature branch:

   ```bash
   git checkout -b feature/amazing-feature
   ```

2. **Make your changes** following the code style guidelines above.

3. **Ensure all tests pass:**

   ```bash
   make test-all
   ```

4. **Ensure code is properly formatted:**

   ```bash
   make fmt-check
   ```

5. **Ensure clippy is clean:**

   ```bash
   make clippy
   ```

6. **Commit your changes** with clear, descriptive commit messages:

   ```bash
   git commit -m "Add amazing feature"
   ```

7. **Push to your fork:**

   ```bash
   git push origin feature/amazing-feature
   ```

8. **Open a Pull Request** against the `main` branch of the upstream repository.

### PR Requirements

- All CI checks must pass
- Code must be formatted (`cargo fmt --all -- --check`)
- Clippy must be clean (`cargo clippy --workspace --all-targets -- -D warnings`)
- All tests must pass
- Add tests for new functionality
- Update documentation if necessary
- Follow the existing code style and conventions

## Issue Reporting

### Bug Reports

When reporting bugs, please include:

- **Description**: Clear description of the issue
- **Steps to Reproduce**: Detailed steps to reproduce the problem
- **Expected Behavior**: What you expected to happen
- **Actual Behavior**: What actually happened
- **Environment**:
  - OS and version
  - Rust version (`rustc --version`)
  - zvec-rust version
  - zvec C library version
- **Logs/Output**: Relevant error messages or logs

### Feature Requests

When requesting features, please include:

- **Description**: Clear description of the proposed feature
- **Use Case**: Why this feature would be useful
- **Alternatives**: Any alternative solutions you've considered
- **Additional Context**: Any other relevant information

## Code Review Standards

When reviewing code, we look for:

- **Correctness**: Does the code work as intended?
- **Safety**: Are there any potential memory safety issues or undefined behavior?
- **Performance**: Is the code efficient? Are there unnecessary allocations or copies?
- **Readability**: Is the code clear and easy to understand?
- **Testing**: Are there adequate tests for the changes?
- **Documentation**: Is the code well-documented?
- **Style**: Does the code follow Rust conventions and project style?

## Keeping in Sync with zvec Core

This SDK tracks the [zvec](https://github.com/alibaba/zvec) C-API. When contributing:

1. Ensure FFI declarations in `zvec-sys/src/lib.rs` match the upstream C-API
2. Add safe wrappers in the `zvec` crate for any new C functions
3. Update integration tests to cover new functionality
4. Run the full test suite to verify compatibility

## Questions?

If you have questions about contributing, feel free to:

- Open an issue with your question
- Reach out via GitHub Discussions

Thank you for contributing to zvec-rust!
