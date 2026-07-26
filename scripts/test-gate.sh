#!/usr/bin/env bash
# Canonical automated development gate for the current Broken Divinity workspace.
set -euo pipefail

export RUST_MIN_STACK="${RUST_MIN_STACK:-16777216}"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
cd "${repo_root}"

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo "========================================="
echo "  Broken Divinity — Development Gate"
echo "========================================="
echo ""

PASS=0
FAIL=0
TEST_LISTED=0
TEST_PASSED=0
TEST_FAILED=0
TEST_IGNORED=0

run_step() {
    local name="$1"
    shift
    local output

    printf '  [%s] ... ' "${name}"
    if output="$("$@" 2>&1)"; then
        printf '%bPASS%b\n' "${GREEN}" "${NC}"
        PASS=$((PASS + 1))
    else
        printf '%bFAIL%b\n' "${RED}" "${NC}"
        printf '%s\n' "${output}" | /usr/bin/tail -40
        FAIL=$((FAIL + 1))
    fi
}

run_test_inventory_step() {
    local output

    printf '  [Test inventory] ... '
    if output="$(cargo test --workspace --locked -- --list 2>&1)"; then
        TEST_LISTED="$(
            printf '%s\n' "${output}" |
                awk '/: test$/ { count += 1 } END { print count + 0 }'
        )"
        printf '%bPASS%b — %d tests listed independently\n' \
            "${GREEN}" "${NC}" "${TEST_LISTED}"
        PASS=$((PASS + 1))
    else
        printf '%bFAIL%b\n' "${RED}" "${NC}"
        printf '%s\n' "${output}" | /usr/bin/tail -80
        FAIL=$((FAIL + 1))
    fi
}

run_test_step() {
    local output
    local metrics
    local observed

    printf '  [Workspace tests] ... '
    if output="$(cargo test --workspace --locked 2>&1)"; then
        printf '%bPASS%b\n' "${GREEN}" "${NC}"
        PASS=$((PASS + 1))
    else
        printf '%bFAIL%b\n' "${RED}" "${NC}"
        printf '%s\n' "${output}" | /usr/bin/tail -80
        FAIL=$((FAIL + 1))
    fi

    metrics="$(
        printf '%s\n' "${output}" |
            awk '
                /test result: / {
                    for (field = 1; field <= NF; field += 1) {
                        if ($field == "passed;") {
                            passed += $(field - 1)
                        } else if ($field == "failed;") {
                            failed += $(field - 1)
                        } else if ($field == "ignored;") {
                            ignored += $(field - 1)
                        }
                    }
                }
                END {
                    printf "%d %d %d", passed, failed, ignored
                }
            '
    )"
    read -r TEST_PASSED TEST_FAILED TEST_IGNORED <<<"${metrics}"
    observed=$((TEST_PASSED + TEST_FAILED + TEST_IGNORED))
    if [[ "${observed}" -ne "${TEST_LISTED}" ]]; then
        printf '    ERROR: listed %d tests but observed %d outcomes\n' \
            "${TEST_LISTED}" "${observed}" >&2
        FAIL=$((FAIL + 1))
    fi
    printf '    measured: %d listed, %d passed, %d failed, %d ignored\n' \
        "${TEST_LISTED}" "${TEST_PASSED}" "${TEST_FAILED}" "${TEST_IGNORED}"
}

report_contract_metrics() {
    local output

    printf '  [Contract metrics] ... '
    if output="$(
        cargo run --quiet --locked -p bd_test_support --bin contract_report -- \
            --registry testing/foundation-contracts.ron \
            --listed "${TEST_LISTED}" \
            --passed "${TEST_PASSED}" \
            --failed "${TEST_FAILED}" \
            --ignored "${TEST_IGNORED}"
    )"; then
        printf '%bPASS%b\n' "${GREEN}" "${NC}"
        printf '%s\n' "${output}" | sed 's/^/    /'
        PASS=$((PASS + 1))
    else
        printf '%bFAIL%b\n' "${RED}" "${NC}"
        printf '%s\n' "${output}" | /usr/bin/tail -40
        FAIL=$((FAIL + 1))
    fi
}

check_ignored_tests() {
    local allowlist="testing/allowed-ignored-tests.txt"
    local -a expected
    local -a actual

    if [[ ! -f "${allowlist}" ]]; then
        printf 'Missing ignored-test allowlist: %s\n' "${allowlist}" >&2
        return 1
    fi

    mapfile -t expected < <(
        awk 'NF && $1 !~ /^#/ { print }' "${allowlist}" | LC_ALL=C sort
    )
    mapfile -t actual < <(
        cargo test --workspace -- --ignored --list 2>/dev/null |
            awk '/: test$/ { sub(/: test$/, ""); print }' |
            LC_ALL=C sort
    )

    if [[ "${expected[*]}" != "${actual[*]}" ]]; then
        printf 'Ignored-test allowlist mismatch.\nExpected:\n' >&2
        printf '  %s\n' "${expected[@]}" >&2
        printf 'Actual:\n' >&2
        printf '  %s\n' "${actual[@]}" >&2
        return 1
    fi

    printf '%d reviewed ignored tests' "${#actual[@]}"
}

if ! command -v cargo >/dev/null 2>&1; then
    printf '%bERROR: cargo is not available on PATH.%b\n' "${RED}" "${NC}"
    exit 1
fi

printf '  [Preflight] RUST_MIN_STACK=%s\n\n' "${RUST_MIN_STACK}"

run_step "Formatting" cargo fmt --all -- --check
run_step "Compile all targets" cargo check --workspace --all-targets --locked
run_step \
    "Contract registry" \
    cargo test -p bd_test_support --test contract_registry --locked
run_step "Ignored-test allowlist" check_ignored_tests
run_test_inventory_step
run_test_step
report_contract_metrics
run_step \
    "Strict Clippy" \
    cargo clippy --workspace --all-targets --locked -- -D warnings
run_step \
    "Content validation" \
    cargo run --quiet --locked -p bd_app -- --validate
run_step "Whitespace" git diff --check

echo ""
echo "========================================="
printf '  Gate steps: %d passed, %d failed\n' "${PASS}" "${FAIL}"
printf '  Tests: %d listed, %d passed, %d failed, %d ignored\n' \
    "${TEST_LISTED}" "${TEST_PASSED}" "${TEST_FAILED}" "${TEST_IGNORED}"
echo "========================================="

if [[ "${FAIL}" -gt 0 ]]; then
    printf '%bGATE FAILED — work is not complete%b\n' "${RED}" "${NC}"
    exit 1
fi

printf '%bAUTOMATED GATE PASSED — complete required GDD and player-facing reviews%b\n' \
    "${GREEN}" "${NC}"
