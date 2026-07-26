#!/usr/bin/env bash
# Release smoke test for Broken Divinity Kernel
set -euo pipefail

echo "=== Release Smoke Test ==="

# Step 1: Release build
echo "--- Step 1: Release build ---"
cargo build --workspace --release --locked
echo "✅ Release build succeeded"

# Step 2: Quick smoke run (send q, expect clean exit)
echo "--- Step 2: Smoke run ---"
if ! command -v script >/dev/null 2>&1; then
    echo "ERROR: util-linux 'script' is required for the PTY smoke test" >&2
    exit 1
fi
printf 'q' | timeout 5 script -qec "./target/release/bd" /dev/null
echo "✅ App runs and exits"

# Step 3: Canonical development gate
echo "--- Step 3: Canonical gate ---"
bash scripts/test-gate.sh
echo "✅ Canonical gate passed"

echo ""
echo "=== Release Smoke Test PASSED ==="
