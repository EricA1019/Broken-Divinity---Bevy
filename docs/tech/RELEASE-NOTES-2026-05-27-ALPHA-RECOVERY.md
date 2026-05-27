# Release Notes - 2026-05-27 Alpha Recovery

## Summary

Broken Divinity is back to a visible, front-door playable Alpha baseline on the reduced composed runtime.

This recovery package restores the startup path, re-establishes the active Menu -> Colony -> Overworld -> Dungeon loop, records executable Alpha evidence, and closes Alpha signoff on the current artifact set.

## Included In This Recovery

### Runtime recovery

- Restored a visible application window after the prior blank-runtime failure.
- Re-established the active reduced runtime composition in `src/runtime_app.rs`.
- Wired the real menu into the live runtime path.
- Preserved deterministic flow between Menu, Colony, Overworld, and Dungeon.

### Compatibility recovery

- Recovered the minimal compatibility surfaces required by the reduced runtime and policy-owner tests.
- Restored active save/runtime state, recap behavior, readability helpers, help/modal policy, and deterministic overworld weather.

### Alpha evidence and signoff

- Completed executable scenario evidence for S01-S08.
- Converted the AXT-00 scorecard into threshold-bearing metric values.
- Closed AXT-08 with `triage_gate_complete = true`, `metrics_met = true`, and `full_gate_green = true`.
- Added a post-alpha desktop live-smoke walkthrough with real window screenshots.

## Validation Basis

Primary acceptance inputs:

1. `cargo metadata --no-deps --format-version 1`
2. `cargo check`
3. `cargo clippy -- -D warnings`
4. `cargo build --release`
5. sequential active external-target test sweep
6. `cargo test --test ux_baseline_red_harness -- --nocapture`

Primary evidence artifacts:

1. `metrics/AXT-00-scorecard-2026-05-26.md`
2. `docs/tech/AXT-08-ALPHA-READINESS-REPORT-2026-05-26.md`
3. `docs/tech/PLAN-2026-05-27-ALPHA-RECOVERY-EXECUTION.md`
4. `docs/tech/playtest-artifacts/2026-05-27-live-smoke/README.md`

## User-Visible Outcome

- The app now opens visibly.
- The menu is interactive on the active runtime path.
- Primary-action progression through the reduced runtime loop is working.
- Save/load recap surfaces and trust-policy contracts are covered by current acceptance artifacts.

## Known Limitations

1. The acceptance decision still relies on reduced-runtime proxy scoring rather than a literal first-time human GUI study.
2. Full `cargo test` remains unreliable as a single monolithic invocation in this environment because of long silent link phases; the project currently relies on sequential target execution for the effective gate.
3. Live-smoke screenshot timing can lag one frame, so the playtest artifact README is the authoritative mapping for the raw image files.

## Recommended Next Work

1. Preserve the current acceptance artifact set as the Alpha baseline.
2. Use the new live-smoke captures as the visual regression baseline for future runtime cleanup.
3. If a stronger acceptance standard is needed later, add a direct human first-session walkthrough on top of the existing proxy and contract evidence.