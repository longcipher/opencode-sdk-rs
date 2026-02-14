# Default recipe to display help
default:
  @just --list

# Format all code
format:
  rumdl fmt .
  taplo fmt
  cargo +nightly fmt --all

# Auto-fix linting issues
fix:
  rumdl check --fix .

# Run all lints
lint:
  typos
  rumdl check .
  taplo fmt --check
  cargo +nightly fmt --all -- --check
  cargo +nightly clippy --all -- -D warnings
  cargo machete

# Run tests
test:
  cargo test --all-features

# Run tests with coverage
test-coverage:
  cargo tarpaulin --all-features --workspace --timeout 300

# Build entire workspace
build:
  cargo build --workspace

# Check all targets compile
check:
  cargo check --all-targets --all-features

# Check for Chinese characters
check-cn:
  rg --line-number --column "\p{Han}"

# Run Prism-based OpenAPI contract tests
test-prism:
  ./scripts/prism-test.sh

# Full CI check
ci: lint test build

# Publish crates to crates.io
publish:
  cargo publish -p opencode-sdk-rs


# ============================================================
# Frontend (Leptos CSR) Commands
# ============================================================

# Start frontend development server with hot reload
fe-dev:
  cargo leptos watch -p leptos-csr-app

# Build frontend in release mode
fe-build:
  cargo leptos build -p leptos-csr-app --release

# Build frontend in debug mode
fe-build-dev:
  cargo leptos build -p leptos-csr-app

# Serve the built frontend application
fe-serve:
  cargo leptos serve -p leptos-csr-app

# Build WASM only
fe-build-wasm:
  cargo build -p leptos-csr-app --target wasm32-unknown-unknown --release

# ============================================================
# Maintenance & Tools
# ============================================================

# Clean build artifacts
clean:
  cargo clean

# Install all required development tools
setup:
  cargo install cargo-leptos
  cargo install cargo-machete
  cargo install taplo-cli
  cargo install typos-cli
  cargo install leptosfmt

# Generate documentation for the workspace
docs:
  cargo doc --no-deps --open
