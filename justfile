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
    uv run maturin develop --features python

# Build WASM module
build-wasm:
    wasm-pack build --target nodejs --out-dir editors/code/wasm --features wasm --no-default-features

# Build VSCode extension (WASM + TypeScript)
build-vscode: build-wasm
    cd editors/code && bun install && bun run compile

# Package VSCode extension as .vsix
package-vscode: build-vscode
    cd editors/code && bunx @vscode/vsce package --allow-missing-repository

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

# Clean build artifacts
clean:
    cargo clean
    rm -rf editors/code/out editors/code/wasm editors/code/node_modules
