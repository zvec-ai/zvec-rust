# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Ongoing improvements and bug fixes

## [0.3.0] - 2025-01-01

### Added

- Safe Rust FFI bindings for the zvec C library
- Builder pattern API for schema, query, and configuration
- RAII resource management with automatic cleanup via `Drop`
- Comprehensive error handling with detailed error codes
- Five example programs demonstrating core functionality:
  - `basic`: End-to-end workflow
  - `schema_builder`: Schema configurations
  - `vector_search`: Vector query patterns
  - `crud_operations`: Full CRUD operations
  - `config_logging`: Library configuration and logging
- Integration tests covering all public APIs
- Fuzz testing targets for type conversions, schema construction, document operations, and query construction
- Benchmark suite for performance monitoring
- CI/CD pipeline with automated testing on multiple platforms
- Cross-platform support for macOS (ARM64/x86_64), Linux (x86_64/ARM64), and Windows (x86_64 MSVC)
- Auto-build support for zvec C library when not found locally
- Type-safe Rust enums for all C constants with compile-time checks
- Zero-copy optimizations where possible to minimize FFI boundary overhead
