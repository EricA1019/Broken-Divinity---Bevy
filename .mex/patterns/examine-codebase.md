---
name: examine-codebase
description: "Running a full Broken Divinity codebase examination — split the repo into parallel review slices, validate repo health locally, and reconcile findings back into MEX metadata."
triggers:
  - "examine codebase"
  - "full examination"
  - "architecture review"
  - "repo audit"
edges:
  - target: "context/architecture.md"
    condition: "to map the repo into subsystem slices and compare code against the 5-tier design"
  - target: "context/conventions.md"
    condition: "to verify state gating, query handling, and tier boundaries while reviewing findings"
last_updated: 2026-04-10
---

# Examine the Broken Divinity Codebase

## Context

Load `.mex/ROUTER.md`, `context/architecture.md`, and `context/conventions.md` first. For a full audit, also inspect `Cargo.toml`, `src/main.rs`, `src/lib.rs`, `docs/dev-plan.md`, and `docs/tech/architecture.md`.

## Steps

1. **Bootstrap the repo state** — read the router, pattern index, and architecture context before splitting work.
2. **Map the audit into slices** — default slices are: core architecture, dungeon/combat, colony/overworld, UI/save-load, and repo health.
3. **Create SQL todos first** — keep each slice independent unless there is a true dependency.
4. **Dispatch sub-agents in parallel** — use one agent per slice; for repo health, include `cargo build -p broken_divinity`, `cargo test -p broken_divinity`, and `cargo clippy -p broken_divinity -- -W clippy::all`.
5. **Run the health checks locally too** — use the main session for the final build/test/clippy status even if a sub-agent already ran them.
6. **Validate surprising claims directly** — re-read any file that would materially change the final assessment before repeating the claim.
7. **Consolidate by theme** — architecture shape, subsystem coverage, repo health, documentation drift, and highest-value risks.
8. **Close the loop** — reconcile SQL todo state, update stale MEX metadata, and add any new reusable gotchas to this pattern.

## Gotchas

- Background agents do **not** share the main session's SQL database. If an agent reports it finished but the todo is still `in_progress`, update the main-session SQL manually.
- Agent summaries can overstate blockers or miss already-wired features. Validate any surprising claim against the actual file before including it.
- `main.rs` currently has a few ungated UI registrations; do not assume the conventions file perfectly matches the code.
- `.mex/ROUTER.md` and `.mex/context/*` can drift behind the codebase. A codebase audit should check the MEX metadata itself, not just gameplay modules.

## Verify

- [ ] Every review slice has a todo and every todo ends as `done` or explicitly `blocked`
- [ ] `cargo build`, `cargo test`, and `cargo clippy -p broken_divinity -- -W clippy::all` have current recorded results
- [ ] High-impact claims in the final report are backed by file references or direct command output
- [ ] Documentation drift versus code is called out explicitly
- [ ] `.mex/ROUTER.md`, relevant `.mex/context/*`, and `patterns/INDEX.md` are updated if the audit exposed stale guidance

## Debug

- If an agent keeps running, keep reading code locally instead of idling; validate claims directly while you wait.
- If an agent claims it updated SQL but the todo state is unchanged, trust the main-session SQL and repair it manually.
- If build/test/clippy output is too large, inspect the saved output file with `view` instead of rerunning the whole command chain.

## Update Scaffold

- [ ] Update `.mex/ROUTER.md` "Current Project State" if what's working/not built has changed
- [ ] Update any `.mex/context/` files that are now out of date
- [ ] If this is a new task type without a pattern, create one in `.mex/patterns/` and add to `INDEX.md`
