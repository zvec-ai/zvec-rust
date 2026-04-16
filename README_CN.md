# zvec-rust

[![CI](https://github.com/sunhailin-Leo/zvec-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/sunhailin-Leo/zvec-rust/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

[English](README.md) | 中文

安全、地道的 [zvec](https://github.com/alibaba/zvec) 向量数据库 Rust 绑定。

## 特性

- **RAII 资源管理** — 所有 C 资源通过 `Drop` 自动释放
- **Builder 模式** — Schema、查询、配置均提供流式 API
- **类型安全** — 所有 C 常量使用 Rust 枚举，编译期检查
- **完善的错误处理** — 所有 FFI 调用返回 `Result<T>`，包含详细错误码
- **尽可能零拷贝** — 最小化 FFI 边界的数据拷贝
- **预编译库支持** — 自动从 GitHub Releases 下载预编译的 `libzvec_c_api`；高阶用户可通过 `ZVEC_LIB_DIR` 覆盖

## 支持的平台

| 平台 | 架构 | CI 状态 | 备注 |
|------|------|---------|------|
| **macOS** | ARM64 (Apple Silicon) | ✅ Clippy + 测试 | 主要开发平台 |
| **macOS** | x86_64 (Intel) | ✅ Clippy + 测试 | |
| **Linux** | x86_64 | ✅ Clippy + 测试 + 模糊测试 + 覆盖率 + 基准测试 | 完整 CI 覆盖 |
| **Linux** | ARM64 (AArch64) | ✅ Clippy + 测试 + 模糊测试 + 覆盖率 | |
| **Windows** | x86_64 (MSVC) | ✅ Clippy + 测试 | CMake + MSVC 工具链 |

> 动态库文件名因平台而异：`libzvec_c_api.dylib`（macOS）、`libzvec_c_api.so`（Linux）、`zvec_c_api.dll`（Windows）。

## 架构

```
zvec-rust/
├── zvec-sys/    # 底层 FFI 绑定（libzvec_c_api）
├── zvec/        # 安全的高层 Rust 封装
└── fuzz/        # 模糊测试目标
```

- **`zvec-sys`** — 原始 `extern "C"` 声明、不透明指针类型和常量
- **`zvec`** — 安全封装，包含 RAII、Builder、迭代器和地道的 Rust API

## 前置条件

Rust SDK 依赖 zvec C 库（`libzvec_c_api`）。**对于大多数用户，无需手动配置** — 构建脚本会自动从 GitHub Releases 下载预编译库。

### 普通用户（零配置）

直接添加依赖即可 — 构建脚本会自动处理一切：

```toml
[dependencies]
zvec = "0.3"
```

首次构建时，`build.rs` 会自动从 [GitHub Releases](https://github.com/sunhailin-Leo/zvec-rust/releases) 下载适合你平台的预编译 `libzvec_c_api`，并通过 `rpath` 设置库路径。

### 高阶用户（自行编译）

如果你需要自行编译 zvec C 库（例如自定义配置或不支持的平台），设置 `ZVEC_LIB_DIR` 环境变量即可覆盖自动下载：

```bash
# 从源码编译 zvec
git clone https://github.com/alibaba/zvec.git && cd zvec
mkdir -p build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release -DBUILD_C_BINDINGS=ON
make -j$(nproc)

# 指向你的自定义构建
export ZVEC_LIB_DIR=/path/to/zvec/build/lib
```

或者使用内置的 Makefile 进行本地开发：

```bash
make setup        # 安装开发工具 + 初始化 git submodule
make zvec-build   # 从 submodule 构建 zvec C 库
make test-all     # 运行所有测试
```

### 库解析顺序

构建脚本按以下顺序查找 C 库：

1. **`ZVEC_LIB_DIR`** 环境变量（最高优先级）
2. **同级目录**：`../zvec/build/lib`
3. **Git submodule**：`vendor/zvec/build/lib`
4. **Vendor 目录**：`vendor/lib/`
5. **预编译下载**：从 GitHub Releases 自动下载
6. **自动构建**：从源码克隆并编译（需要 `git`、`cmake`、C++17 编译器）

设置 `ZVEC_AUTO_BUILD=0` 可禁用第 5 和第 6 步。

## 快速开始

```rust
use zvec::*;

fn main() -> zvec::Result<()> {
    initialize(None)?;

    // 定义 Schema
    let schema = CollectionSchema::builder("my_collection")
        .add_field(FieldSchema::new("id", DataType::String, false, 0))
        .add_vector_field("embedding", DataType::VectorFp32, 128,
            IndexParams::hnsw(MetricType::Cosine, 16, 200))
        .build()?;

    // 创建集合并插入数据
    let collection = Collection::create_and_open("./data", &schema, None)?;

    let mut doc = Doc::new()?;
    doc.set_pk("doc1");
    doc.add_string("id", "doc1")?;
    doc.add_vector_f32("embedding", &vec![0.1; 128])?;
    collection.insert(&[&doc])?;

    // 向量相似度搜索
    let query = VectorQuery::new("embedding", &vec![0.2; 128], 10)?;
    let results = collection.query(&query)?;
    for result in &results {
        println!("pk={}, score={:.4}", result.get_pk().unwrap_or(""), result.get_score());
    }

    shutdown()?;
    Ok(())
}
```

## 示例

使用 `cargo run --example <名称>` 运行示例：

| 示例 | 说明 |
|---|---|
| `basic` | 端到端流程：schema → 插入 → 查询 → 获取 → 删除 |
| `schema_builder` | 各种 Schema 配置：字段类型、索引类型、量化 |
| `vector_search` | 向量查询模式：简单查询、Builder、过滤器、输出字段、HNSW 参数 |
| `crud_operations` | 完整 CRUD：插入、获取、更新、upsert、删除、统计、刷新 |
| `config_logging` | 库配置：内存限制、线程数、日志 |

```bash
cargo run --example basic
cargo run --example vector_search
```

## API 概览

### 初始化

| 函数 | 说明 |
|---|---|
| `initialize(config)` | 初始化库（调用一次） |
| `shutdown()` | 释放所有资源 |
| `version()` | 获取版本字符串 |
| `is_initialized()` | 检查初始化状态 |

### Schema 定义

```rust
let schema = CollectionSchema::builder("name")
    .add_field(FieldSchema::new("field", DataType::String, false, 0))
    .add_vector_field("vec", DataType::VectorFp32, 128,
        IndexParams::hnsw(MetricType::Cosine, 16, 200))
    .build()?;
```

### 集合操作

| 方法 | 说明 |
|---|---|
| `Collection::create_and_open()` | 创建新集合 |
| `Collection::open()` | 打开已有集合 |
| `collection.insert(&docs)` | 插入文档 |
| `collection.update(&docs)` | 更新文档 |
| `collection.upsert(&docs)` | 插入或更新 |
| `collection.delete(&pks)` | 按主键删除 |
| `collection.query(&query)` | 向量相似度搜索 |
| `collection.fetch(&pks)` | 按主键获取 |
| `collection.stats()` | 获取集合统计信息 |
| `collection.flush()` | 刷新到磁盘 |

### 文档操作

```rust
let mut doc = Doc::new()?;
doc.set_pk("my_pk");
doc.add_string("name", "value")?;
doc.add_i64("count", 42)?;
doc.add_vector_f32("embedding", &[0.1, 0.2, 0.3])?;

let name = doc.get_string("name")?;
let count = doc.get_i64("count")?;
```

### 向量查询

```rust
// 简单查询
let query = VectorQuery::new("embedding", &query_vec, 10)?;

// Builder 模式 + 过滤器
let query = VectorQuery::builder()
    .field_name("embedding")
    .vector(&query_vec)
    .topk(10)
    .filter("category = 'tech'")
    .output_fields(&["id", "name"])
    .build()?;
```

## 支持的类型

| 类别 | 类型 |
|---|---|
| **标量** | `Bool`, `Int32`, `Int64`, `Uint32`, `Uint64`, `Float`, `Double`, `String`, `Binary` |
| **向量** | `VectorFp16`, `VectorFp32`, `VectorFp64`, `VectorInt4`, `VectorInt8`, `VectorInt16`, `VectorBinary32`, `VectorBinary64` |
| **稀疏向量** | `SparseVectorFp16`, `SparseVectorFp32` |
| **数组** | `ArrayBool`, `ArrayInt32`, `ArrayInt64`, `ArrayFloat`, `ArrayDouble`, `ArrayString` |

## 索引类型

| 类型 | 构造函数 | 说明 |
|---|---|---|
| HNSW | `IndexParams::hnsw(metric, m, ef)` | 图索引（推荐） |
| HNSW+量化 | `IndexParams::hnsw_with_quantize(...)` | 带量化的 HNSW |
| IVF | `IndexParams::ivf(metric, nlist, niters, soar)` | 倒排文件索引 |
| Flat | `IndexParams::flat(metric)` | 暴力搜索索引 |
| Invert | `IndexParams::invert(range, wildcard)` | 标量字段索引 |

## 测试

```bash
# 使用 Makefile（推荐 — 自动检测库路径）
make test-unit         # 单元测试（不需要 C 库）
make test-integration  # 集成测试（需要 C 库）
make test-all          # 所有测试（单元 + 集成 + 文档）

# 直接使用 cargo（需要设置 ZVEC_LIB_DIR / DYLD_LIBRARY_PATH）
cargo test --lib
cargo test --test integration_test

# 模糊测试（需要 nightly）
cargo install cargo-fuzz
cargo +nightly fuzz run fuzz_types -- -max_total_time=60

# 性能基准测试
make bench

# 代码覆盖率
cargo install cargo-llvm-cov
./scripts/coverage.sh --html
```

## 与 zvec 核心保持同步

本 SDK 跟踪 [zvec](https://github.com/alibaba/zvec) 的 C-API。当上游 C-API 发生变更时：

1. 更新 `zvec-sys/src/lib.rs` 中的 FFI 声明
2. 在 `zvec` crate 中添加安全封装
3. 更新集成测试以覆盖新功能
4. 运行完整测试套件验证兼容性

CI 流水线会自动克隆最新的 zvec 并构建 C 库，确保每个 PR 的 FFI 兼容性。

## 贡献

1. Fork 本仓库
2. 创建功能分支（`git checkout -b feature/amazing-feature`）
3. 确保所有测试通过（`cargo test`）
4. 确保代码格式正确（`cargo fmt --all -- --check`）
5. 确保 clippy 无警告（`cargo clippy --workspace --all-targets -- -D warnings`）
6. 提交 Pull Request

## 许可证

Apache-2.0 — 详见 [LICENSE](LICENSE)。
