# Phase 0 Cleanup Baseline - 2026-05-29

## Keep

- tracked docs, handoffs, metrics, and playtest artifacts
- prototype binaries and reference docs until an explicit retirement phase says otherwise
- broken_divinity_save.json as the persistence baseline

## Remove

- ignored local build outputs under target/
- transient logs matched by *.log
- temporary captures created for the current work when they are not tracked evidence

## Ignore Policy Status

- Root ignore policy created on 2026-05-29.
- Current disposable artifact rules: /target/ and *.log.
- Manual dev walkthrough command: cargo run --bin broken_divinity --features dev
- Rollback runtime command: cargo run --bin broken_divinity

## Notes

- This baseline does not authorize deletion of tracked docs or tracked playtest artifacts.
- This baseline does not treat broken_divinity_save.json as cleanup clutter.