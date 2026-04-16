# zvec-rust

[![CI](https://github.com/sunhailin-Leo/zvec-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/sunhailin-Leo/zvec-rust/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

English | [中文](README_CN.md)

Safe, idiomatic Rust bindings for the [zvec](https://github.com/alibaba/zvec) vector database.

## Features

- **RAII Resource Management** — All C resources are automatically freed via `Drop`
- **Builder Pattern** — Fluent APIs for schema, query, and configuration
- **Type Safety** — Rust enums for all C constants with compile-time checks
- **Comprehensive Error Handling** — All FFI calls return `Result<T>` with detailed error codes
- **Zero-Copy Where Possible** — Minimizes data copying across the FFI boundary
- **Prebuilt Libraries** — Automatically downloads prebuilt `libzvec_c_api` from GitHub Releases; advanced users can override with `ZVEC_LIB_DIR`

## Supported Platforms

| Platform | Architecture | CI Status | Notes |
|----------|-------------|-----------|-------|
| **macOS** | ARM64 (Apple Silicon) | ✅ Clippy + Test | Primary development platform |
| **macOS** | x86_64 (Intel) | ✅ Clippy + Test | |
| **Linux** | x86_64 | ✅ Clippy + Test + Fuzz + Coverage + Benchmark | Full CI coverage |
| **Linux** | ARM64 (AArch64) | ✅ Clippy + Test + Fuzz + Coverage | |
| **Windows** | x86_64 (MSVC) | ✅ Clippy + Test | CMake + MSVC toolchain |

> The dynamic library name varies by platform: `libzvec_c_api.dylib` (macOS), `libzvec_c_api.so` (Linux), `zvec_c_api.dll` (Windows).

## Architecture

```
zvec-rust/
├── zvec-sys/    # Low-level FFI bindings to libzvec_c_api
├── zvec/        # Safe, high-level Rust wrapper
└── fuzz/        # Fuzz testing targets
```

- **`zvec-sys`** — Raw `extern "C"` declarations, opaque pointer types, and constants
- **`zvec`** — Safe wrappers with RAII, builders, iterators, and idiomatic Rust APIs

## Prerequisites

The Rust SDK depends on the zvec C library (`libzvec_c_api`). **For most users, no manual setup is needed** — the build script automatically downloads a prebuilt library from GitHub Releases.

### For Regular Users (Zero Setup)

Just add the dependency — the build script handles everything:

```toml
[dependencies]
zvec = "0.3"
```

On first build, `build.rs` will automatically download the prebuilt `libzvec_c_api` for your platform from [GitHub Releases](https://github.com/sunhailin-Leo/zvec-rust/releases) and set up the library path via `rpath`.

### For Advanced Users (Custom Build)

If you want to build the zvec C library yourself (e.g., for a custom configuration or unsupported platform), set the `ZVEC_LIB_DIR` environment variable to override the automatic download:

```bash
# Build zvec from source
git clone https://github.com/alibaba/zvec.git && cd zvec
mkdir -p build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release -DBUILD_C_BINDINGS=ON
make -j$(nproc)

# Point zvec-rust to your custom build
export ZVEC_LIB_DIR=/path/to/zvec/build/lib
```

Or use the built-in Makefile for local development:

```bash
make setup        # Install dev tools + init git submodule
make zvec-build   # Build the zvec C library from submodule
make test-all     # Run all tests
```

### Library Resolution Order

The build script resolves the C library in this order:

1. **`ZVEC_LIB_DIR`** environment variable (highest priority)
2. **Sibling checkout**: `../zvec/build/lib`
3. **Git submodule**: `vendor/zvec/build/lib`
4. **Vendor directory**: `vendor/lib/`
5. **Prebuilt download**: from GitHub Releases (automatic)
6. **Auto-build**: clone and build from source (requires `git`, `cmake`, C++17 compiler)

Set `ZVEC_AUTO_BUILD=0` to disable steps 5 and 6.

## Quick Start

```rust
use zvec::*;

fn main() -> zvec::Result<()> {
    initialize(None)?;

    // Define schema
    let schema = CollectionSchema::builder("my_collection")
        .add_field(FieldSchema::new("id", DataType::String, false, 0))
        .add_vector_field("embedding", DataType::VectorFp32, 128,
            IndexParams::hnsw(MetricType::Cosine, 16, 200))
        .build()?;

    // Create collection and insert data
    let collection = Collection::create_and_open("./data", &schema, None)?;

    let mut doc = Doc::new()?;
    doc.set_pk("doc1");
    doc.add_string("id", "doc1")?;
    doc.add_vector_f32("embedding", &vec![0.1; 128])?;
    collection.insert(&[&doc])?;

    // Vector similarity search
    let query = VectorQuery::new("embedding", &vec![0.2; 128], 10)?;
    let results = collection.query(&query)?;
    for result in &results {
        println!("pk={}, score={:.4}", result.get_pk().unwrap_or(""), result.get_score());
    }

    shutdown()?;
    Ok(())
}
```

## Examples

Run any example with `cargo run --example <name>`:

| Example | Description |
|---|---|
| `basic` | End-to-end workflow: schema → insert → query → fetch → delete |
| `schema_builder` | Various schema configurations: field types, index types, quantization |
| `vector_search` | Vector query patterns: simple, builder, filter, output fields, HNSW params |
| `crud_operations` | Full CRUD: insert, fetch, update, upsert, delete, stats, flush |
| `config_logging` | Library configuration: memory limits, thread counts, logging |

```bash
cargo run --example basic
cargo run --example vector_search
```

## API Overview

### Initialization

| Function | Description |
|---|---|
| `initialize(config)` | Initialize the library (call once) |
| `shutdown()` | Release all resources |
| `version()` | Get version string |
| `is_initialized()` | Check initialization status |

### Schema Definition

```rust
let schema = CollectionSchema::builder("name")
    .add_field(FieldSchema::new("field", DataType::String, false, 0))
    .add_vector_field("vec", DataType::VectorFp32, 128,
        IndexParams::hnsw(MetricType::Cosine, 16, 200))
    .build()?;
```

### Collection Operations

| Method | Description |
|---|---|
| `Collection::create_and_open()` | Create a new collection |
| `Collection::open()` | Open an existing collection |
| `collection.insert(&docs)` | Insert documents |
| `collection.update(&docs)` | Update documents |
| `collection.upsert(&docs)` | Insert or update |
| `collection.delete(&pks)` | Delete by primary keys |
| `collection.query(&query)` | Vector similarity search |
| `collection.fetch(&pks)` | Fetch by primary keys |
| `collection.stats()` | Get collection statistics |
| `collection.flush()` | Flush to disk |

### Document Operations

```rust
let mut doc = Doc::new()?;
doc.set_pk("my_pk");
doc.add_string("name", "value")?;
doc.add_i64("count", 42)?;
doc.add_vector_f32("embedding", &[0.1, 0.2, 0.3])?;

let name = doc.get_string("name")?;
let count = doc.get_i64("count")?;
```

### Vector Query

```rust
// Simple query
let query = VectorQuery::new("embedding", &query_vec, 10)?;

// Builder pattern with filters
let query = VectorQuery::builder()
    .field_name("embedding")
    .vector(&query_vec)
    .topk(10)
    .filter("category = 'tech'")
    .output_fields(&["id", "name"])
    .build()?;
```

## Supported Types

| Category | Types |
|---|---|
| **Scalar** | `Bool`, `Int32`, `Int64`, `Uint32`, `Uint64`, `Float`, `Double`, `String`, `Binary` |
| **Vector** | `VectorFp16`, `VectorFp32`, `VectorFp64`, `VectorInt4`, `VectorInt8`, `VectorInt16`, `VectorBinary32`, `VectorBinary64` |
| **Sparse** | `SparseVectorFp16`, `SparseVectorFp32` |
| **Array** | `ArrayBool`, `ArrayInt32`, `ArrayInt64`, `ArrayFloat`, `ArrayDouble`, `ArrayString` |

## Index Types

| Type | Constructor | Description |
|---|---|---|
| HNSW | `IndexParams::hnsw(metric, m, ef)` | Graph index (recommended) |
| HNSW+Q | `IndexParams::hnsw_with_quantize(...)` | HNSW with quantization |
| IVF | `IndexParams::ivf(metric, nlist, niters, soar)` | Inverted file index |
| Flat | `IndexParams::flat(metric)` | Brute-force index |
| Invert | `IndexParams::invert(range, wildcard)` | Scalar field index |

## Testing

```bash
# Using Makefile (recommended — auto-detects library paths)
make test-unit         # Unit tests (no C library required)
make test-integration  # Integration tests (requires C library)
make test-all          # All tests (unit + integration + doc)

# Using cargo directly (requires ZVEC_LIB_DIR / DYLD_LIBRARY_PATH)
cargo test --lib
cargo test --test integration_test

# Fuzz tests (requires nightly)
cargo install cargo-fuzz
cargo +nightly fuzz run fuzz_types -- -max_total_time=60

# Benchmarks
make bench

# Code coverage
cargo install cargo-llvm-cov
./scripts/coverage.sh --html
```

## Keeping in Sync with zvec Core

This SDK tracks the [zvec](https://github.com/alibaba/zvec) C-API. When the upstream C-API changes:

1. Update `zvec-sys/src/lib.rs` with new FFI declarations
2. Add safe wrappers in the `zvec` crate
3. Update integration tests to cover new functionality
4. Run the full test suite to verify compatibility

The CI pipeline automatically clones the latest zvec and builds the C library, ensuring FFI compatibility on every PR.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Ensure all tests pass (`cargo test`)
4. Ensure code is formatted (`cargo fmt --all -- --check`)
5. Ensure clippy is clean (`cargo clippy --workspace --all-targets -- -D warnings`)
6. Submit a pull request

## License

Apache-2.0 — see [LICENSE](LICENSE).
