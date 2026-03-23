# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Husk is a fast beancount file formatter with CJK double-width character support. It provides a Rust library, CLI binary, Python bindings (via PyO3/maturin, managed by uv), and WASM bindings (via wasm-bindgen). The VSCode extension uses bun for Node package management.

## Build & Test Commands

All commands are available via `just` (see `justfile` for details):

```bash
just                    # List all available recipes
just build              # Build library
just build-cli          # Build CLI binary
just build-python       # Build Python extension (uv + maturin)
just build-wasm         # Build WASM module
just build-vscode       # Build VSCode extension (WASM + TypeScript)
just package-vscode     # Package .vsix
just test               # Run all tests
just test-file name     # Run a specific test file
just clippy             # Run clippy lints
just fmt                # Format code
just fmt-check          # Check formatting
just check              # Run all checks (fmt, clippy, test)
just clean              # Clean build artifacts
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
