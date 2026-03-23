# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Husk is a fast beancount file formatter with CJK double-width character support. It provides a Rust library, CLI binary, Python bindings (via PyO3/maturin), and WASM bindings (via wasm-bindgen).

## Build & Test Commands

```bash
# Build (library only, no feature flags)
cargo build

# Build CLI (default feature)
cargo build                     # cli feature is default
cargo build --features cli      # explicit

# Build Python extension
maturin develop --features python

# Build WASM
cargo build --features wasm --target wasm32-unknown-unknown

# Run all tests
cargo test

# Run a single test file
cargo test --test align_test
cargo test --test integration_test

# Run a specific test by name
cargo test test_name_substring

# Clippy
cargo clippy --all-targets --all-features

# Format check
cargo fmt -- --check
```

## Architecture

The formatting pipeline in `lib.rs::format()` processes input line-by-line through three stages:

1. **Sort** (`sort.rs`) — optionally reorder entries by date
2. **Parse** (`line.rs`) — regex-based parser classifies each line into a `Line` enum variant (TransactionHeader, Posting, Balance, Open, Close, Price, MetaItem, Comment, etc.)
3. **Normalize + Align** — per-variant formatting:
   - `normalize.rs` — standardizes indentation, thousands separators, brace spacing, comment formatting
   - `align.rs` — column-aligns currencies and costs using `unicode-width` for CJK-aware display width calculation

Key design points:

- `Options` struct (`options.rs`) is the single configuration object shared across all targets
- `recursive.rs` (gated behind `cli`/`python` features) — BFS traversal of `include` directives with glob expansion
- Features `python` and `wasm` are **mutually exclusive** (enforced by compile_error!)
- All regex patterns use `LazyLock` for one-time compilation
- The `line.rs` parser uses zero-copy `&str` slices via the `Line<'a>` enum
