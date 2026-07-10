#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TIMEOUT_SECONDS=8
EXPECTED_TIMEOUT_EXIT=124

run_profile_smoke() {
  local profile_flag="$1"

  set +e
  timeout "${TIMEOUT_SECONDS}s" cargo run --quiet -- --headless "${profile_flag}" >/tmp/bd-qa-log-smoke.out 2>&1
  local exit_code=$?
  set -e

  if [[ ${exit_code} -ne ${EXPECTED_TIMEOUT_EXIT} ]]; then
    echo "QA smoke failed for ${profile_flag}. Expected timeout exit ${EXPECTED_TIMEOUT_EXIT}, got ${exit_code}." >&2
    echo "--- Captured output ---" >&2
    cat /tmp/bd-qa-log-smoke.out >&2
    exit 1
  fi

  echo "QA smoke passed for ${profile_flag} (startup sustained for ${TIMEOUT_SECONDS}s)."
}

cd "${ROOT_DIR}"
run_profile_smoke "--qa-standard"
run_profile_smoke "--qa-deep-diagnostics"

echo "All QA log smoke checks passed."
