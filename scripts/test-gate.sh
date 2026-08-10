#!/usr/bin/env bash
# Canonical automated development gate for the current Broken Divinity workspace.
set -euo pipefail

export RUST_MIN_STACK="${RUST_MIN_STACK:-16777216}"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
cd "${repo_root}"

CANDIDATE_MANIFEST=""
CANDIDATE_MANIFEST_SHA256=""
while [[ "$#" -gt 0 ]]; do
    case "$1" in
        --candidate-manifest)
            CANDIDATE_MANIFEST="${2:-}"
            shift 2
            ;;
        --manifest-sha256)
            CANDIDATE_MANIFEST_SHA256="${2:-}"
            shift 2
            ;;
        *)
            printf 'Unknown test-gate argument: %s\n' "$1" >&2
            exit 2
            ;;
    esac
done

if [[ -n "${CANDIDATE_MANIFEST}" || -n "${CANDIDATE_MANIFEST_SHA256}" ]]; then
    if [[ -z "${CANDIDATE_MANIFEST}" || -z "${CANDIDATE_MANIFEST_SHA256}" ]]; then
        printf 'Candidate mode requires --candidate-manifest and --manifest-sha256 together.\n' >&2
        exit 2
    fi
    GATE_MODE="candidate"
else
    GATE_MODE="canonical"
fi

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
CANDIDATE_CONTRACTS=()
CANDIDATE_PROTECTION_ARGS=(
    --require-protected AGENTS.md
    --require-protected GDD.md
    --require-protected Kernel.md
    --require-protected docs/authority/DECISIONS-TO-LOCK.md
    --require-protected Cargo.toml
    --require-protected scripts/test-gate.sh
    --require-protected testing/allowed-ignored-tests.txt
    --require-protected testing/foundation-contracts.ron
    --require-protected testing/FOUNDATION-TEST-EVIDENCE.md
    --require-protected testing/FOUNDATION-REQUIREMENT-MAP.md
    --require-protected testing/VISUAL-ACCEPTANCE-MATRIX.md
    --require-protected docs/authority/AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md
    --require-protected docs/active/FOUNDATION-STABILIZATION-AND-CONSOLE-HARDENING-PLAN.md
    --require-protected crates/bd_test_support/Cargo.toml
    --require-protected crates/bd_test_support/src/lib.rs
    --require-protected crates/bd_test_support/src/contract_registry.rs
    --require-protected crates/bd_test_support/src/bin/handoff_guard.rs
    --require-protected crates/bd_test_support/src/bin/contract_report.rs
    --require-protected crates/bd_test_support/tests/candidate_handoff.rs
    --require-protected crates/bd_test_support/tests/contract_registry.rs
    --require-protected crates/bd_test_support/tests/repository_governance.rs
)

check_candidate_handoff() {
    cargo run --quiet --locked -p bd_test_support --bin handoff_guard -- \
        --root "${repo_root}" \
        --manifest "${CANDIDATE_MANIFEST}" \
        --manifest-sha256 "${CANDIDATE_MANIFEST_SHA256}" \
        "${CANDIDATE_PROTECTION_ARGS[@]}"
}

load_candidate_contracts() {
    local output
    output="$(
        cargo run --quiet --locked -p bd_test_support --bin handoff_guard -- \
            --root "${repo_root}" \
            --manifest "${CANDIDATE_MANIFEST}" \
            --manifest-sha256 "${CANDIDATE_MANIFEST_SHA256}" \
            "${CANDIDATE_PROTECTION_ARGS[@]}" \
            --print-contracts
    )" || return 1
    mapfile -t CANDIDATE_CONTRACTS <<<"${output}"
    [[ "${#CANDIDATE_CONTRACTS[@]}" -gt 0 ]]
}

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
    local -a command

    printf '  [Contract metrics] ... '
    command=(
        cargo run --quiet --locked -p bd_test_support --bin contract_report --
        --registry testing/foundation-contracts.ron
        --listed "${TEST_LISTED}"
        --passed "${TEST_PASSED}"
        --failed "${TEST_FAILED}"
        --ignored "${TEST_IGNORED}"
    )
    if [[ "${GATE_MODE}" == "candidate" ]]; then
        for contract in "${CANDIDATE_CONTRACTS[@]}"; do
            command+=(--candidate-contract "${contract}")
        done
    fi
    if output="$("${command[@]}")"; then
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

if [[ "${GATE_MODE}" == "candidate" ]]; then
    if ! load_candidate_contracts; then
        printf '%bSTATUS=NotComplete — candidate handoff manifest is invalid or changed%b\n' \
            "${RED}" "${NC}"
        exit 1
    fi
    printf '  [Candidate handoff] protected manifest valid; contracts: %s\n\n' \
        "${CANDIDATE_CONTRACTS[*]}"
fi

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
if [[ "${GATE_MODE}" == "candidate" ]]; then
    run_step "Handoff integrity" check_candidate_handoff
fi

echo ""
echo "========================================="
printf '  Gate steps: %d passed, %d failed\n' "${PASS}" "${FAIL}"
printf '  Tests: %d listed, %d passed, %d failed, %d ignored\n' \
    "${TEST_LISTED}" "${TEST_PASSED}" "${TEST_FAILED}" "${TEST_IGNORED}"
echo "========================================="

if [[ "${FAIL}" -gt 0 ]]; then
    printf '%bSTATUS=NotComplete — %s gate failed; focused green cannot waive this result%b\n' \
        "${RED}" "${GATE_MODE}" "${NC}"
    exit 1
fi

if [[ "${GATE_MODE}" == "candidate" ]]; then
    printf '%bSTATUS=CandidateGreen — implementation gates passed; protected authority remains Red for independent review%b\n' \
        "${GREEN}" "${NC}"
    exit 0
fi

printf '%bSTATUS=VerifiedGreen — automated gate passed; ReviewedGreen still requires diff, GDD, evidence, and player-facing review%b\n' \
    "${GREEN}" "${NC}"
