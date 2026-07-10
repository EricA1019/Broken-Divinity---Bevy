#!/usr/bin/env bash
# =============================================================================
# Broken Divinity — Testing Gate
#
# Canonical entry point for build, test, clippy, and release checks.
# Run from the project root:  bash scripts/test-gate.sh
#
# Requirements:
#   - RUST_MIN_STACK must be >= 16777216 (required by rustc for Bevy dependency
#     graph on some Linux configurations). The script sets it automatically.
#   - cargo must be on PATH.
# =============================================================================
set -euo pipefail

# ── Environment setup ──────────────────────────────────────────────────────
# Required: rustc can SIGSEGV on large Bevy dependency graphs without this.
export RUST_MIN_STACK="${RUST_MIN_STACK:-16777216}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "========================================="
echo "  Broken Divinity — Testing Gate"
echo "========================================="
echo ""

PASS=0
FAIL=0

run_step() {
    local name="$1"
    shift
    echo -n "  [$name] ... "
    if output=$("$@" 2>&1); then
        echo -e "${GREEN}PASS${NC}"
        PASS=$((PASS + 1))
    else
        echo -e "${RED}FAIL${NC}"
        echo "$output" | /usr/bin/tail -20
        FAIL=$((FAIL + 1))
    fi
}

# ── Preflight: verify required tooling ─────────────────────────────────────
if ! command -v cargo &>/dev/null; then
    echo -e "${RED}ERROR: 'cargo' not found on PATH. Ensure Rust toolchain is installed.${NC}"
    exit 1
fi

echo "  [Preflight] RUST_MIN_STACK=${RUST_MIN_STACK}"
echo ""

# ── Step 1: Clean build ────────────────────────────────────────────────────
run_step "Build (debug)" cargo build -p broken_divinity

# ── Step 2: All tests ──────────────────────────────────────────────────────
run_step "Tests" cargo test -p broken_divinity

# ── Step 3: Clippy lint ────────────────────────────────────────────────────
# Allow known Bevy-isms: complex query types and many system params
run_step "Clippy" cargo clippy -p broken_divinity -- -W clippy::all \
    -A clippy::too_many_arguments -A clippy::type_complexity

# ── Step 4: Release build (catches optimization-only issues) ───────────────
run_step "Build (release)" cargo build -p broken_divinity --release

echo ""
echo "========================================="
echo "  Results: ${PASS} passed, ${FAIL} failed"
echo "========================================="

if [ "$FAIL" -gt 0 ]; then
    echo -e "${RED}GATE FAILED — do not distribute${NC}"
    exit 1
else
    echo -e "${GREEN}GATE PASSED — ready for testing${NC}"
    exit 0
fi
