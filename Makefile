.PHONY: all build build-release check clean clippy doc fmt fmt-check \
       test test-unit test-integration test-doc test-all \
       bench coverage coverage-html coverage-lcov coverage-text \
       fuzz fuzz-types fuzz-schema fuzz-doc fuzz-query fuzz-collection \
       example example-basic example-schema example-search example-crud example-config \
       lint setup help \
       submodule-init submodule-update zvec-build zvec-clean zvec-rebuild

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

CARGO          := cargo
CARGO_FUZZ     := cargo +nightly fuzz
FUZZ_SECONDS   ?= 60
COVERAGE_DIR   := coverage-html
ZVEC_SUBMODULE := vendor/zvec
ZVEC_BUILD_DIR := $(ZVEC_SUBMODULE)/build
CMAKE_JOBS     ?= $(shell sysctl -n hw.ncpu 2>/dev/null || nproc 2>/dev/null || echo 2)

# Auto-detect zvec C library path (submodule build → sibling checkout → env var)
ZVEC_LIB_DIR   ?= $(shell \
  if [ -d "$(ZVEC_BUILD_DIR)/lib" ]; then echo "$(CURDIR)/$(ZVEC_BUILD_DIR)/lib"; \
  elif [ -d "../zvec/build/lib" ]; then cd ../zvec/build/lib && pwd; \
  fi)

# Set the platform-appropriate dynamic library search path
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
  export DYLD_LIBRARY_PATH := $(ZVEC_LIB_DIR):$(DYLD_LIBRARY_PATH)
else
  export LD_LIBRARY_PATH := $(ZVEC_LIB_DIR):$(LD_LIBRARY_PATH)
endif
export ZVEC_LIB_DIR

# ---------------------------------------------------------------------------
# Default target
# ---------------------------------------------------------------------------

all: fmt-check clippy test-unit  ## Run format check, clippy, and unit tests

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

build:  ## Build in debug mode
	$(CARGO) build --workspace

build-release:  ## Build in release mode
	$(CARGO) build --workspace --release

check:  ## Type-check without producing binaries
	$(CARGO) check --workspace --all-targets

clean:  ## Remove build artifacts
	$(CARGO) clean
	rm -rf $(COVERAGE_DIR) lcov.info coverage.json benchmark-output.txt

# ---------------------------------------------------------------------------
# Format & Lint
# ---------------------------------------------------------------------------

fmt:  ## Format all code
	$(CARGO) fmt --all

fmt-check:  ## Check formatting without modifying files
	$(CARGO) fmt --all -- --check

clippy:  ## Run clippy lints
	$(CARGO) clippy --workspace --all-targets -- -D warnings

lint: fmt-check clippy  ## Run all linters (fmt + clippy)

# ---------------------------------------------------------------------------
# Test
# ---------------------------------------------------------------------------

test: test-unit  ## Run unit tests (alias)

test-unit:  ## Run unit tests (no C library required)
	$(CARGO) test --workspace --lib -- --nocapture

test-integration:  ## Run integration tests (requires C library)
	$(CARGO) test --workspace --test '*' -- --nocapture

test-doc:  ## Run documentation tests
	$(CARGO) test --workspace --doc

test-all: test-unit test-integration test-doc  ## Run all tests (unit + integration + doc)

# ---------------------------------------------------------------------------
# Benchmark
# ---------------------------------------------------------------------------

bench:  ## Run benchmarks
	$(CARGO) bench --workspace

bench-compile:  ## Compile benchmarks without running
	$(CARGO) bench --workspace --no-run

# ---------------------------------------------------------------------------
# Coverage
# ---------------------------------------------------------------------------

coverage: coverage-html  ## Generate coverage report (alias for coverage-html)

coverage-html:  ## Generate HTML coverage report
	./scripts/coverage.sh --html

coverage-lcov:  ## Generate LCOV coverage report
	./scripts/coverage.sh --lcov

coverage-text:  ## Print text coverage summary
	./scripts/coverage.sh --text

coverage-json:  ## Generate JSON coverage report
	./scripts/coverage.sh --json

# ---------------------------------------------------------------------------
# Fuzz Testing (requires nightly toolchain)
# ---------------------------------------------------------------------------

fuzz: fuzz-types fuzz-schema fuzz-doc fuzz-query fuzz-collection  ## Run all fuzz targets

fuzz-types:  ## Fuzz type conversions
	$(CARGO_FUZZ) run fuzz_types -- -max_total_time=$(FUZZ_SECONDS)

fuzz-schema:  ## Fuzz schema construction
	$(CARGO_FUZZ) run fuzz_schema -- -max_total_time=$(FUZZ_SECONDS)

fuzz-doc:  ## Fuzz document operations
	$(CARGO_FUZZ) run fuzz_doc -- -max_total_time=$(FUZZ_SECONDS)

fuzz-query:  ## Fuzz query construction
	$(CARGO_FUZZ) run fuzz_query -- -max_total_time=$(FUZZ_SECONDS)

fuzz-collection:  ## Fuzz collection CRUD operations
	$(CARGO_FUZZ) run fuzz_collection -- -max_total_time=$(FUZZ_SECONDS)

# ---------------------------------------------------------------------------
# Examples
# ---------------------------------------------------------------------------

example: example-basic  ## Run the basic example (alias)

example-basic:  ## Run basic example
	$(CARGO) run --example basic

example-schema:  ## Run schema_builder example
	$(CARGO) run --example schema_builder

example-search:  ## Run vector_search example
	$(CARGO) run --example vector_search

example-crud:  ## Run crud_operations example
	$(CARGO) run --example crud_operations

example-config:  ## Run config_logging example
	$(CARGO) run --example config_logging

# ---------------------------------------------------------------------------
# Documentation
# ---------------------------------------------------------------------------

doc:  ## Generate and open API documentation
	$(CARGO) doc --workspace --no-deps --open

doc-build:  ## Generate API documentation (no open)
	$(CARGO) doc --workspace --no-deps

# ---------------------------------------------------------------------------
# zvec C Library (via git submodule)
# ---------------------------------------------------------------------------

submodule-init:  ## Initialize the zvec git submodule
	@if git submodule status $(ZVEC_SUBMODULE) 2>/dev/null | grep -q '^'; then \
		echo "📦 Updating existing zvec submodule..."; \
		git submodule update --init --recursive; \
	elif [ -f "$(ZVEC_SUBMODULE)/CMakeLists.txt" ]; then \
		echo "📦 zvec submodule already present."; \
	else \
		echo "📦 Adding zvec submodule for the first time..."; \
		git submodule add https://github.com/alibaba/zvec.git $(ZVEC_SUBMODULE); \
		git submodule update --init --recursive; \
	fi

submodule-update:  ## Update the zvec git submodule to latest
	git submodule update --remote --merge

zvec-build: submodule-init  ## Build the zvec C library from submodule
	@if [ ! -f "$(ZVEC_SUBMODULE)/CMakeLists.txt" ]; then \
		echo "Error: zvec submodule not found. Run 'make submodule-init' first."; \
		exit 1; \
	fi
	@mkdir -p $(ZVEC_BUILD_DIR)
	@echo "🔧 [1/2] Configuring zvec with CMake..."
	@cd $(ZVEC_BUILD_DIR) && cmake .. \
		-DCMAKE_BUILD_TYPE=Release \
		-DCMAKE_POLICY_VERSION_MINIMUM=3.5 \
		-DBUILD_C_BINDINGS=ON \
		-DBUILD_TOOLS=OFF
	@echo "🔨 [2/2] Building zvec C library ($(CMAKE_JOBS) parallel jobs)..."
	cd $(ZVEC_BUILD_DIR) && cmake --build . --config Release -j $(CMAKE_JOBS)
	@echo ""
	@echo "✅ zvec C library built successfully at $(ZVEC_BUILD_DIR)/lib"
	@echo "   You can now run: make build / make test-all"

zvec-clean:  ## Remove zvec C library build artifacts
	rm -rf $(ZVEC_BUILD_DIR)

zvec-rebuild: zvec-clean zvec-build  ## Clean and rebuild the zvec C library

# ---------------------------------------------------------------------------
# Setup & Dependencies
# ---------------------------------------------------------------------------

setup: submodule-init  ## Install development dependencies and init submodule
	rustup component add rustfmt clippy llvm-tools-preview
	cargo install cargo-llvm-cov cargo-fuzz
	@echo ""
	@echo "✅ Development tools installed and submodule initialized."
	@echo ""
	@echo "To build the zvec C library (required for integration tests):"
	@echo "  make zvec-build"
	@echo ""
	@echo "Or set environment variables manually:"
	@echo "  export ZVEC_LIB_DIR=/path/to/zvec/build/lib"
	@echo "  export ZVEC_INCLUDE_DIR=/path/to/zvec/src/include"

# ---------------------------------------------------------------------------
# CI (reproduces the full CI pipeline locally)
# ---------------------------------------------------------------------------

ci: lint test-all bench-compile  ## Run the full CI pipeline locally

ci-full: lint test-all bench  ## Run the full CI pipeline with benchmarks

# ---------------------------------------------------------------------------
# Help
# ---------------------------------------------------------------------------

help:  ## Show this help message
	@echo "zvec-rust Makefile"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Environment variables:"
	@echo "  ZVEC_LIB_DIR       Path to libzvec_c_api (for integration tests)"
	@echo "  ZVEC_INCLUDE_DIR   Path to zvec headers"
	@echo "  FUZZ_SECONDS       Fuzz duration per target (default: 60)"
	@echo "  CMAKE_JOBS         Parallel jobs for cmake build (default: auto)"
	@echo ""
	@echo "Quick start for development:"
	@echo "  make setup          # Install tools + init submodule"
	@echo "  make zvec-build     # Build the zvec C library"
	@echo "  make test-all       # Run all tests"
