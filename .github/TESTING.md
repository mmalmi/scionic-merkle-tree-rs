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
# All tests (Go interop tests auto-skip if Go repo not available)
cargo test --verbose

# Clippy lints
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check
```

**Note:** Tests automatically detect if the Go repository is available at `/workspace/Scionic-Merkle-Tree` and skip Go interop tests if it's not found. This means you can safely run `cargo test` anywhere - tests will adapt based on what's available.

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

1. **test-basic** - All tests (Go interop auto-skipped)
   - Runs on stable & beta Rust
   - Go interop tests skip if Go repo unavailable
   - Caches cargo registry/build
   - All core tests run

2. **test-full** - Complete test suite with Go interop
   - Runs on stable Rust
   - Checks out Go implementation first
   - All tests including Go interop run

3. **clippy** - Linting checks
   - Runs on stable Rust
   - Fails on any warnings (-D warnings)

4. **fmt** - Format checks
   - Runs on stable Rust
   - Ensures consistent code formatting

**Key feature:** Tests automatically detect dependencies and skip gracefully. No need to maintain skip lists!

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
