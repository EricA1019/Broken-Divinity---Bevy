# AXT-08 Alpha Readiness Report (2026-05-26)

## Scope
Authoritative report for AXT-08 signoff readiness decision.

## Gate Summary
1. Compile/check/lint/release gates: PASS
2. Effective test gate across all active external targets: PASS
3. Defect triage executable gate logic: PASS (`tests/axt07_alpha_battery.rs`)
4. Signoff decision logic: PASS (`tests/axt08_alpha_signoff.rs`)

Gate evidence delta (2026-05-27):
1. Reduced runtime composition is active through `src/runtime_app.rs`.
2. Menu flow is wired into the live reduced runtime path.
3. Full `cargo test` as a single uncontended invocation still enters a long silent link phase in this environment, so the test gate was completed through sequential execution of every active external target.
4. A post-alpha front-door desktop smoke run captured real window evidence in `docs/tech/playtest-artifacts/2026-05-27-live-smoke/README.md`.

## Metric Status
1. Contract-level policy and flow tests: PASS
2. Executable scenario evidence for S01-S08: PASS via `tests/ux_baseline_red_harness.rs`
3. Numeric threshold scoring: PASS via `metrics/AXT-00-scorecard-2026-05-26.md`
4. `metrics_met = true` under the criteria-based proxy scoring method already established for this environment
5. Post-alpha live-smoke confidence upgrade: PASS via the desktop walkthrough artifact set in `docs/tech/playtest-artifacts/2026-05-27-live-smoke/README.md`

## Risks
1. Alpha signoff is now based on a reduced-runtime proxy methodology rather than a direct human GUI walkthrough.
2. If the project later requires literal first-time human-session evidence, that would be a stronger post-alpha validation artifact rather than a current blocker.
3. The live-smoke capture pipeline can lag a frame, so the artifact README is the authoritative state-to-file mapping for the raw screenshots.

## Signoff Decision
- Current decision: ACCEPTED
- Signoff input:
	- `triage_gate_complete = true`
	- `metrics_met = true`
	- `full_gate_green = true`
- Rationale: all three inputs required by `src/alpha_signoff.rs` now resolve true on the current reduced-runtime evidence set.

## Required Next Actions
1. Preserve the reduced-runtime scorecard and signoff artifacts as the Alpha acceptance baseline.
2. Treat any future direct-human walkthrough as a post-alpha confidence upgrade, not an acceptance blocker.
3. Keep the live-smoke artifact set with the acceptance package so later regressions can be compared against a real window baseline.
