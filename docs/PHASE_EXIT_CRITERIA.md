# Phase Exit Criteria

## Phase 0 — Dependency and Runtime Compatibility Gate

- [x] App compiles (`cargo check --workspace`)
- [x] Terminal opens (manual smoke — `cargo run -p bd_app`)
- [x] Frame draws (manual smoke)
- [x] Key input reaches Bevy (manual smoke — press q to quit)
- [x] Quit exits cleanly (manual smoke)
- [ ] Terminal restores after normal exit
- [ ] Terminal restores after panic/early exit
- [x] Dependency compatibility matrix exists
- [x] Cargo.lock is committed
- [x] Dependency versions are pinned
- [x] DecisionLog.md exists
- [x] Basic CI/check script exists (justfile)
- [x] Failure policy is documented
- [x] color-eyre vs anyhow decision is made

## Phase 1 — Minimal Terminal Slice

- [ ] @ moves
- [ ] walls block movement
- [ ] help line exists
- [ ] stat panel exists
- [ ] log panel exists
- [ ] build/version footer exists
- [ ] first ASCII snapshot test exists
- [ ] temporary glyph/color fixture usage is documented

## Phase 2 — PoolDelta Core

- [ ] Health/AP use same pipeline
- [ ] No separate DamageSystem/HealingSystem exists
- [ ] All pool mutation goes through PoolDelta
- [ ] Pool changes appear in trace/log
