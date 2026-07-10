---
name: agents
description: Always-loaded project anchor. Read this first. Contains project identity, non-negotiables, commands, and pointer to ROUTER.md for full context.
last_updated: 2026-04-21
---

# Broken Divinity

## What This Is
A post-apocalyptic religious horror roguelike RPG built in Rust with Bevy 0.18, featuring hybrid ASCII/sprite rendering, d100 skill-based combat, dual sanity systems, and a walkable Rimworld-style shelter colony.

## Non-Negotiables
- Never unwrap queries in systems — all query access must use `let Ok(...) = ... else { return; }` or equivalent graceful handling
- Never import from a higher module tier — Tier 0 (core) through Tier 5 (orchestration) is a strict one-way dependency graph
- Never use Bevy Events API — use Messages (`#[derive(Message)]`, `MessageWriter<T>`, `Messages<T>`, `add_message::<T>()`)
- Always gate systems to an `AppState` and/or `TurnState` — no ungated Update systems
- Never delete decisions.md entries — mark as superseded

## Commands
- Build: `cargo build -p broken_divinity`
- Run: `cargo run -p broken_divinity`
- Test: `cargo test -p broken_divinity`
- Lint: `cargo clippy -p broken_divinity -- -W clippy::all`
- Prune: `scripts/prune-build-artifacts.sh` (add `--dry-run` to preview first)
- Drift check: `mex check --quiet`

## Scaffold Growth
After every task: if no pattern exists for the task type you just completed, create one. If a pattern or context file is now out of date, update it. The scaffold grows from real work, not just setup. See the GROW step in `ROUTER.md` for details.

## Navigation
At the start of every session, read `ROUTER.md` before doing anything else.
For full project context, patterns, and task guidance — everything is there.
