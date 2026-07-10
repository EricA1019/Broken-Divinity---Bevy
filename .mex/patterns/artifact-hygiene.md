---
name: artifact-hygiene
description: Keep Cargo outputs and other generated artifacts from growing without bound while preserving the most useful runnable binaries.
triggers:
  - "clean build artifacts"
  - "prune target"
  - "repo is too large"
  - "build directory is huge"
  - "stale outputs"
edges:
  - target: "context/conventions.md"
    condition: when changing maintenance scripts or other repo workflow helpers
  - target: "context/setup.md"
    condition: when documenting maintenance commands for contributors
last_updated: 2026-04-21
---

# Artifact Hygiene

## Context

- This repo's `target/` tree can balloon after repeated debug/release builds and
  long Bevy iteration cycles.
- Cargo can regenerate object files, dependency outputs, incremental state, and
  generated docs, so those are better cleanup targets than the current top-level
  binaries.
- The hygiene goal is to reclaim space aggressively without throwing away the
  newest runnable `broken_divinity` binaries unless the user explicitly wants a
  full clean.

## Canonical helper

- Use `./scripts/prune-build-artifacts.sh`

## Default policy

1. Keep top-level binaries in `target/debug` and `target/release`
2. Remove heavyweight rebuildable state:
   - `deps`
   - `incremental`
   - `build`
   - `.fingerprint`
   - generated docs
   - `examples`
   - `test-artifacts`
   - `cxxbridge`
3. Prefer `--dry-run` first when auditing space pressure
4. Use `--full` only when you intentionally want an almost-clean build tree

## When to run it

- After large build/test loops
- After switching repeatedly between debug and release
- Before archiving or snapshotting the repo workspace
- Any time `target/` looks disproportionate to the current task

## Verify

- Run a shell syntax check on `scripts/prune-build-artifacts.sh`
- Run the helper with the `--dry-run` flag before deleting anything
- Measure `target/` before and after pruning

## Gotchas

- This helper is intentionally narrower than `cargo clean`; default mode keeps
  the current top-level binaries in place for quick reruns.
- If you need every compiled artifact gone, use `--full` or `cargo clean`.
