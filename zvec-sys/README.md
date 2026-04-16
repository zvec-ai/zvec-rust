# zvec-sys

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](../LICENSE)

Raw FFI bindings to the [zvec](https://github.com/alibaba/zvec) C-API (`libzvec_c_api`).

## Overview

This crate provides low-level, unsafe Rust bindings to the zvec vector database C library. It is intended to be used by the higher-level [`zvec`](https://crates.io/crates/zvec) crate, which provides safe, idiomatic Rust APIs.

**You probably want to use the [`zvec`](https://crates.io/crates/zvec) crate instead.**

## Contents

- Opaque pointer types for all zvec C objects (collections, documents, schemas, queries, etc.)
- `extern "C"` function declarations matching the zvec C-API
- Constants for error codes, data types, index types, metric types, and more

## Prerequisites

This crate requires the zvec C library (`libzvec_c_api`) to be available at link time. The build script (`build.rs`) resolves the library location in the following order:

1. **Environment variables**: `ZVEC_LIB_DIR` and `ZVEC_INCLUDE_DIR`
2. **Sibling directory**: `../zvec/build/lib`
3. **Vendor directory**: `vendor/lib`
4. **Auto-build** (experimental): Clones and builds zvec from source

See the [main README](../README.md) for detailed setup instructions.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
zvec-sys = { path = "../zvec-sys" }
```

All functions are `unsafe` and require careful handling of raw pointers and C memory management. Prefer the safe `zvec` crate for application code.

## License

Apache-2.0 — see [LICENSE](../LICENSE).
