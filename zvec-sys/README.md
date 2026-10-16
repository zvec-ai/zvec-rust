# zvec-rust-sys

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](../LICENSE)

Raw FFI bindings to the [zvec](https://github.com/alibaba/zvec) C-API (`libzvec_c_api`).

## Overview

This crate provides low-level, unsafe Rust bindings to the zvec vector database C library. It is intended to be used by the higher-level [`zvec-rust`](https://crates.io/crates/zvec-rust) crate, which provides safe, idiomatic Rust APIs.

**You probably want to use the [`zvec-rust`](https://crates.io/crates/zvec-rust) crate instead.**

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

Set `ZVEC_STRICT_LIB=1` to accept step 1 only: the library must then come from a
`ZVEC_LIB_DIR` that contains the platform library, and the build script fails
with the expected file name instead of falling back to steps 2–4, a cached or
downloaded prebuilt SDK, or a source build. See the
[main README](../README.md) for details.

See the [main README](../README.md) for detailed setup instructions.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
zvec-rust-sys = { path = "../zvec-sys" }
```

All functions are `unsafe` and require careful handling of raw pointers and C memory management. Prefer the safe `zvec-rust` crate for application code.

## License

Apache-2.0 — see [LICENSE](../LICENSE).
