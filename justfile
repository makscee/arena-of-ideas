set shell := ["bash", "-cu"]

[private]
default:
    just --list

# Build all crates
build:
    cargo build -p shared -p client
    cargo build -p server --target wasm32-unknown-unknown

# Run all unit tests
test:
    cargo test -p shared -p client --lib

# Run integration tests (requires local SpacetimeDB server)
test-integration:
    cargo test -p client --test harness -- --test-threads=1

# Run all tests
test-all: test test-integration

# Check formatting and lints
check:
    cargo fmt --check
    cargo clippy -p shared -p client -- -D warnings
    cargo clippy -p server --target wasm32-unknown-unknown -- -D warnings

# Format code
fmt:
    cargo fmt

# Publish server module to local SpacetimeDB
publish-local:
    spacetime publish -p server --server local aoi-test --delete-data -y

# Publish and run integration tests
publish-test: publish-local test-integration

# Run the game client
run:
    cargo run -p client

# Build client for WASM
build-wasm:
    cargo build -p client --target wasm32-unknown-unknown

# Full CI check (everything except integration tests)
ci: check test build-wasm
