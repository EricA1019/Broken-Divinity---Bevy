# Bevy BRP Smoke Checklist

Short, reusable smoke checks for live debugging with Bevy Remote Protocol (BRP).

## 1) Start In Dev Mode

Run the game with BRP enabled:

```bash
cargo run -p broken_divinity --features dev
```

Expected:
- Game launches successfully.
- BRP HTTP transport binds to port `15702`.

## 2) Verify BRP Port

From another terminal:

```bash
ss -ltn '( sport = :15702 )'
curl -sS -m 2 http://127.0.0.1:15702/
```

Expected:
- `ss` shows a listener on `127.0.0.1:15702` (or `0.0.0.0:15702`).
- `curl` returns an HTTP response (content may vary by BRP server version).

## 3) MCP Connectivity Check

Verify MCP server registration:

```bash
copilot mcp list
copilot mcp get bevy-brp
```

Expected:
- `bevy-brp` appears in the server list.
- Command path resolves to your BRP MCP binary.

## 4) Core Debug Smoke Targets

Use BRP/MCP tools to validate these core runtime checks after a gameplay change:

- App state transitions work: `Menu -> Colony -> Overworld -> Dungeon -> Colony`.
- Player entity exists in active state and has expected core components.
- Critical resources are readable and non-default when expected (time, logs, travel/raid state).
- Mutation path works for one safe value change and readback (resource or component).
- No duplicate/stale entities after scene transitions.

## 4a) Stale-Entity Safety Sequence (Required)

When validating BRP behavior around scene transitions, use this sequence:

1. Query/select entity IDs only after entering the target state.
2. Re-validate the entity before read/mutate calls.
3. If validation fails, refresh selection instead of retrying blindly.

Expected stale-entity behavior:
- No panic in game process.
- Structured diagnostics with code `brp.stale_entity`.
- Diagnostic includes operation context and a refresh hint.

Interpretation:
- `brp.stale_entity` is a recoverable QA path issue, not a gameplay runtime crash.
- Re-run entity query in current state and continue smoke checks.

## 5) Regression Pairing (Required)

Always pair BRP smoke with automated tests:

```bash
cargo test -p broken_divinity
```

For release readiness, also run:

```bash
scripts/test-gate.sh
```

## 6) Quick Failure Triage

If BRP is not reachable:

1. Confirm game is running in `--features dev` mode.
2. Confirm no port conflict on `15702`.
3. Re-check `bevy-brp` MCP config (`copilot mcp get bevy-brp`).
4. Restart the game process and retry port probe.

If BRP returns stale-entity diagnostics:

1. Confirm entity came from a prior state or pre-despawn snapshot.
2. Re-query entities in the current state.
3. Continue smoke checks only after entity refresh succeeds.

## 7) Evidence To Capture

For each smoke run, capture:

- Commit SHA.
- Scenario tested (feature/bugfix name).
- BRP port verification output.
- 3-5 key BRP observations (state, entities, resources).
- Test result summary (`cargo test` and/or `scripts/test-gate.sh`).
