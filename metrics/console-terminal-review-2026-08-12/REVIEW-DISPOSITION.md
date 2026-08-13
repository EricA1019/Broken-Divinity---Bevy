# Console Real-Terminal Review — 2026-08-12

Independent reviewer disposition: **PASS / ReviewedGreen** with two defects
fixed (input-capture leak and multi-line output flattening). Owner acceptance
is not claimed. The final canonical gate reports `STATUS=VerifiedGreen`
(10/10 steps, 841/841 tests).

## Method

The `bd` binary (`cargo build --locked -p bd_app`) was independently driven
through a real PTY at `80x24` and `60x20`, with an in-place
`80x24 -> 60x20` resize, using crossterm raw-mode input (backtick, printable
characters, `Enter`, and `Esc`). Terminal output was rendered through a `pyte`
screen and captured to text files under `evidence/`. All three PTY stderr logs
were empty.

## Checklist results

| Behavior | 80x24 | 60x20 | Resize | Evidence |
|---|---|---|---|---|
| Input capture and no gameplay replay | PASS | PASS | PASS | `80x24_open_console_after_gameplay_bound_e.txt`, `60x20_3_stats.txt`, `60x20_resize_help.txt` |
| Open/resize/close | PASS | PASS | PASS | `resize_1_60x20_console_open.txt`, `60x20_resize_closed_clean.txt` |
| Overlay legibility | PASS | PASS | PASS | `80x24_3_stats.txt`, `60x20_3_stats.txt` |
| Command output | PASS | PASS | PASS | `80x24_3_stats.txt`, `80x24_4_day.txt` |
| Debug mutation (`day 42`) | PASS | — | — | `80x24_4_day.txt`, `80x24_5_closed.txt` |
| Clean-frame restoration | PASS | PASS | PASS | `60x20_clean_before_console.txt` and `60x20_resize_closed_clean.txt` are byte-identical; direct close captures also contain no overlay cells |

## Defects found and fixed

### 1. Console-typed keys leaked into gameplay (blocker)

While the console was open, typed characters were **both** captured by the
console and reprocessed by gameplay on a later frame — e.g. typing `e` opened
the Station Staffing modal and typing `stats` triggered Travel (`t`), even
though the console buffer and output showed the command.

Root cause: `map_input_to_intents` returned early when
`ConsoleState.batch_capture_active` was set but did not drain its
`MessageReader<KeyMessage>`. When the capture flag cleared on a following
frame, the lagging gameplay reader reprocessed the same keys.

Fix: `crates/bd_tui/src/lib.rs` — drain the reader before the early return.

Regression test:
`console_input_contract::open_console_typed_key_does_not_leak_to_gameplay_on_a_later_frame`.
The first version wrote the message before `App::update` and falsely passed
when `messages.clear()` was removed. Independent review repaired the observer
to emit during `PreUpdate`, matching the real terminal adapter. With only the
drain removed, the repaired test fails with turn `0 -> 1`; with the drain
present it passes.

Verified in the real terminal: a lone `e` now stays in the console buffer and
no modal opens; `stats` no longer triggers Travel.

### 2. Multi-line command output was flattened (minor)

`stats`/`blueprints`/`events` produce newline-joined output, but the console
renderer treated each output entry as a single `Line`, collapsing embedded
newlines into one wrapped paragraph (`... Faith: 0/100Survivors: ...`).

Fix: `crates/bd_console/src/render.rs` — split each output entry on `\n`
before constructing the wrapped lines.

Regression test: `console_render_contract_tests::multi_line_command_output_preserves_logical_rows`.

Independent mutation review replaced newline splitting with the old one-line
path; the test failed with `day: 0turn: 1` on one row, then passed after the
production implementation was restored.

Verified in the real terminal: `stats` now renders each field on its own row.

## Records

- Files changed: `crates/bd_tui/src/lib.rs`, `crates/bd_console/src/render.rs`,
  `crates/bd_app/tests/console_input_contract.rs` (one regression test),
  `crates/bd_tui/src/console_render_contract_tests.rs` (one regression test).
- Canonical gate (post-fix): `STATUS=VerifiedGreen`, 841/841 tests, 0 Red.
- GDD/Kernel drift review: no feature, balance, content, terminal-profile,
  debug-boundary, renderer-owner, or input-owner expansion was introduced.
- Registry remains 127 `GreenUnreviewed`, distinguishing reviewer completion
  from owner acceptance.
