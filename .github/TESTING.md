# Testing CI Locally

This guide explains how to test the GitHub Actions CI workflows locally before pushing.

## Method 1: Run CI Commands Directly (Recommended)

Use the provided script to run all CI test commands:

```bash
./.github/test-ci-locally.sh
```

This script runs:
1. **Basic tests** (no Go interop)
2. **Clippy** (lints)
3. **Rustfmt** (format check)
4. **Full tests** (with Go interop, requires Go repo)

### Manual Commands

You can also run individual test suites:

```bash
# Basic tests (what CI runs on stable/beta Rust)
cargo test --verbose -- --skip go_compatibility --skip interop --skip chunk_size_interop

# Clippy lints
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check

# Full tests with Go interop (requires Go repo checked out at ../Scionic-Merkle-Tree)
cargo test --verbose
```

## Method 2: Use `act` to Simulate GitHub Actions

[`act`](https://github.com/nektos/act) runs GitHub Actions workflows locally using Docker.

### Install act

```bash
# macOS
brew install act

# Linux
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Windows (via Chocolatey)
choco install act-cli
```

### Run Workflows

```bash
# Run all jobs
act

# Run specific job
act -j test-basic       # Basic tests only
act -j clippy           # Clippy lints only
act -j fmt              # Format check only
act -j test-full        # Full tests with Go interop

# List available jobs
act -l
```

### Notes on `act`

- First run will prompt to select a Docker image (choose "Medium" for best compatibility)
- Requires Docker to be running
- Some GitHub-specific features may not work identically
- The test script (Method 1) is faster and more reliable for quick checks

## CI Workflow Jobs

Our CI has 4 jobs:

1. **test-basic** - Fast tests without external dependencies
   - Runs on stable & beta Rust
   - Skips Go interop tests
   - Caches cargo registry/build

2. **test-full** - Complete test suite
   - Runs on stable Rust
   - Includes Go interop tests
   - Checks out Go implementation

3. **clippy** - Linting checks
   - Runs on stable Rust
   - Fails on any warnings (-D warnings)

4. **fmt** - Format checks
   - Runs on stable Rust
   - Ensures consistent code formatting

## Troubleshooting

### "Go repo not found"

The full tests require the Go implementation. Clone it:

```bash
cd ..
git clone https://github.com/HORNET-Storage/Scionic-Merkle-Tree
cd Scionic-Merkle-Tree
go build -o scionic-merkle-tree ./cmd
cd ../scionic-merkle-tree-rs
```

### Clippy warnings

Fix all warnings:

```bash
cargo clippy --fix --allow-dirty --allow-staged
```

### Format issues

Auto-format the code:

```bash
cargo fmt
```

### act Docker issues

If `act` fails with Docker errors, try:

```bash
# Use a specific Docker image
act -P ubuntu-latest=catthehacker/ubuntu:act-latest

# Increase verbosity
act -v
```
