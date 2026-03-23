# Default recipe: list available commands
default:
    @just --list

# Build the library (no feature flags)
build:
    cargo build

# Build the CLI binary
build-cli:
    cargo build --features cli

# Build Python extension via uv + maturin
build-python:
    uvx maturin develop --features python

# Build WASM module
build-wasm:
    wasm-pack build --target nodejs --out-dir editors/code/wasm --features wasm --no-default-features

# Build VSCode extension (WASM + TypeScript)
build-vscode: build-wasm
    cd editors/code && bun install && bun run compile

# Package VSCode extension as .vsix
package-vscode: build-vscode
    cd editors/code && bunx @vscode/vsce package

# Run all tests
test:
    cargo test

# Run a specific test file
test-file name:
    cargo test --test {{ name }}

# Run clippy lints
clippy:
    cargo clippy --all-targets --all-features

# Check formatting
fmt-check:
    cargo fmt -- --check

# Format code
fmt:
    cargo fmt

# Run all checks (fmt, clippy, test)
check: fmt-check clippy test

# Publish crate to crates.io
publish-crate: check
    cargo publish

# Publish Python package to PyPI (set UV_PUBLISH_TOKEN)
publish-python:
    uvx maturin build --release --features python
    uv publish target/wheels/*.whl

# Publish VSCode extension to Marketplace
publish-vscode: package-vscode
    cd editors/code && bunx @vscode/vsce publish

# Publish all packages
publish-all: publish-crate publish-python publish-vscode

# Clean build artifacts
clean:
    cargo clean
    rm -rf editors/code/out editors/code/wasm editors/code/node_modules
