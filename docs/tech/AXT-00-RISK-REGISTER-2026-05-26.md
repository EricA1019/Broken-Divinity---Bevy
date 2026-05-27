# AXT-00 Risk Register (2026-05-26)

## Scope
AXT-00 evidence-only baseline risk register.

## Risks

1. ID: AXT00-R01
- Severity: P1
- Area: Runtime flow execution
- Description: Required scripted flows are executable at contract level, but full gameplay runtime wiring for end-to-end scenario execution is still incomplete.
- Owner: runtime-recovery
- Exit criteria: Runtime wiring restored to support Menu->Colony->Overworld->Dungeon->Return and save/load path execution with live systems.
- Progress evidence:
	- `tests/runtime_flow_contracts.rs` (Menu->Colony->Overworld->Dungeon->Return and save/load roundtrip contracts)
	- `src/runtime_flow.rs`

2. ID: AXT00-R02
- Severity: RESOLVED
- Area: Policy ownership
- Description: Intended owner files for instruction hierarchy, modal/escape, and feedback policy were missing.
- Owner: architecture-recovery
- Exit criteria: RCV-06A/06B/06C recovery subtasks completed and owner modules established.
- Resolution evidence:
	- `src/ui/objective_prompt.rs`
	- `src/ui/modal_priority.rs`
	- `src/core/escape.rs`
	- `src/core/gamelog.rs`

3. ID: AXT00-R03
- Severity: P2
- Area: UX metric validity
- Description: Scorecard has N/A scenario values due non-runnable runtime paths.
- Owner: ux-harness
- Exit criteria: Execute scripted scenario battery and replace N/A with measured results.

## Baseline Gate Status
- Compile/test/lint/release gates: PASS
- UX baseline runtime scenario gates: BLOCKED

## Disposition
AXT-00 started and evidence artifacts created, but AXT-00 cannot be marked complete until runtime flow scenarios are executable.
