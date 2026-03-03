tarpaulin := "cargo tarpaulin --config tarpaulin.toml --all-features"

# Show all tasks
[private]
default:
  just -l

# Run all checks (fmt, clippy, test)
check: fmt clippy test

# Check formatting
fmt:
  cargo fmt --all -- --check

# Run clippy
clippy:
  cargo clippy --workspace --all-features -- -D warnings

# Run tests
test:
  cargo test --workspace --all-features

# Generate coverage report (tarpaulin in Docker; broken on Apple Silicon due to QEMU+Rust SIGSEGV)
coverage:
  #!/usr/bin/env bash
  if [[ "$(uname -s)" == "Darwin" && "$(uname -m)" == "arm64" ]]; then
    echo "⚠ Coverage via Docker tarpaulin is broken on Apple Silicon (QEMU SIGSEGV with Rust ≥1.85)."
    echo "  Coverage runs in CI via cargo-llvm-cov. See .github/workflows/test.yml"
    exit 1
  fi
  mkdir -p coverage
  {{tarpaulin}}

# Run coverage in docker (for x86_64 Linux or Intel Mac)
docker-coverage:
  docker run \
    --rm \
    --pull always \
    --volume .:/volume \
    --workdir /volume \
    --platform linux/amd64 \
    --security-opt seccomp=unconfined \
    xd009642/tarpaulin \
    {{tarpaulin}}

# Build release
build:
  cargo build --release

# Clean build artifacts
clean:
  cargo clean
  rm -rf coverage
