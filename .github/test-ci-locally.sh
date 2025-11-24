#!/bin/bash
# Test CI commands locally before pushing

set -e

echo "========================================="
echo "Testing CI Commands Locally"
echo "========================================="
echo

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Test basic tests
echo "=== Test 1: Basic Tests (Go interop tests auto-skip if Go repo unavailable) ==="
echo "Command: cargo test --verbose"
echo

cargo test --verbose && \
echo -e "${GREEN}✓ Basic tests passed${NC}" || \
(echo -e "${RED}✗ Basic tests failed${NC}" && exit 1)

echo
echo "=== Test 2: Clippy (Lints) ==="
echo "Command: cargo clippy -- -D warnings"
echo

cargo clippy -- -D warnings && \
echo -e "${GREEN}✓ Clippy passed${NC}" || \
(echo -e "${RED}✗ Clippy failed${NC}" && exit 1)

echo
echo "=== Test 3: Rustfmt (Format Check) ==="
echo "Command: cargo fmt -- --check"
echo

cargo fmt -- --check && \
echo -e "${GREEN}✓ Format check passed${NC}" || \
(echo -e "${RED}✗ Format check failed (run 'cargo fmt' to fix)${NC}" && exit 1)

echo
echo "=== Test 4: Full Tests (With Go, requires Go repo) ==="
echo "Command: cargo test --verbose"
echo "Note: This requires the Go implementation at ../Scionic-Merkle-Tree"
echo

if [ ! -d "../Scionic-Merkle-Tree" ]; then
  echo -e "${RED}✗ Go repo not found. Skipping full tests.${NC}"
  echo "To test full CI, clone the Go repo:"
  echo "  cd .."
  echo "  git clone https://github.com/HORNET-Storage/Scionic-Merkle-Tree"
  echo "  cd Scionic-Merkle-Tree && go build -o scionic-merkle-tree ./cmd"
  exit 0
fi

export PATH="../Scionic-Merkle-Tree:$PATH"
cargo test --verbose && \
echo -e "${GREEN}✓ Full tests passed${NC}" || \
(echo -e "${RED}✗ Full tests failed${NC}" && exit 1)

echo
echo "========================================="
echo -e "${GREEN}All CI tests passed!${NC}"
echo "========================================="
