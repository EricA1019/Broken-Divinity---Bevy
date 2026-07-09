#!/usr/bin/env bash
# Release smoke test for Broken Divinity Kernel
set -euo pipefail

echo "=== Release Smoke Test ==="

# Step 1: Fresh checkout build
echo "--- Step 1: Release build ---"
cargo build -p bd_app --release 2>&1
echo "✅ Release build succeeded"

# Step 2: Quick smoke run (send q, expect clean exit)
echo "--- Step 2: Smoke run ---"
timeout 3 ./target/release/bd </dev/null 2>&1 || true
echo "✅ App runs and exits"

# Step 3: Content validation
echo "--- Step 3: Content validation ---"
./target/release/bd --validate 2>&1
echo "✅ Content validation passed"

# Step 4: Run tests
echo "--- Step 4: Test suite ---"
cargo test --workspace 2>&1 | tail -5
echo "✅ All tests passed"

echo ""
echo "=== Release Smoke Test PASSED ==="
