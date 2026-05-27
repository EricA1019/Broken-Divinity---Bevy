# Live Smoke Report - 2026-05-27

## Scope

Post-alpha front-door smoke run against the local desktop binary to upgrade confidence beyond deterministic proxy tests.

This report does not replace the Alpha acceptance artifacts in `metrics/` and `docs/tech/AXT-08-ALPHA-READINESS-REPORT-2026-05-26.md`.
It adds one real-window walkthrough on top of the accepted reduced-runtime baseline.

## Environment

- Host: Linux desktop with X11 available
- Binary: `./target/debug/broken_divinity`
- Launch surface: direct binary execution, not test harness playback
- Input method: X11 window activation plus keyboard events
- Capture method: `xwd` raw window dumps converted to PNG with `ffmpeg`

## Journey Summary

| Step | Input | Expected | Observed | Artifact | Status |
| --- | --- | --- | --- | --- | --- |
| Cold start | Launch binary | Visible menu | Real menu window opened with title, seed field, and New Game / Load Game / Quit actions | `menu.png` | PASS |
| Start run | `Enter` / `Space` | Transition into Colony | Colony state rendered with `Travel to Overworld` as the primary CTA and low-risk recap text | `colony-current.png` | PASS |
| Advance to overworld | `Enter` / `Space` | Transition into Overworld | Overworld state rendered with `Enter Dungeon` as the primary CTA and medium-risk recap text | `overworld.png` | PASS |
| Advance to dungeon | `Enter` / `Space` | Transition into Dungeon | Dungeon state rendered with `Return to Colony` as the primary CTA and high-risk recap text | `colony-return.png` | PASS |
| Pause check | `Escape` | Pause or modal feedback visible | The captured frame remained on Colony with no visible pause overlay before capture completed | `colony-paused.png` | PARTIAL |

## Artifact Mapping Notes

The raw screenshot filenames are not a perfect step-by-step timeline.
Capture timing lagged one transition in part of the sequence, so this report is the authoritative mapping for the artifacts.

Verified canonical screenshots:

- `menu.png`: Menu
- `colony-current.png`: Colony
- `overworld.png`: Overworld
- `colony-return.png`: Dungeon with `Return to Colony` CTA

Raw captures retained for audit/debugging:

- `colony.png`
- `dungeon.png`
- `.xwd` source dumps for every PNG

## Findings

1. The application opens a real visible window and reaches the main menu reliably.
2. The active reduced runtime supports the expected front-door loop: Menu -> Colony -> Overworld -> Dungeon.
3. The on-screen controls panel matches the reduced runtime keyboard affordances observed during the walkthrough.
4. State summaries remain readable in live rendering, including recap risk and next-step guidance.
5. The pause/modal capture is inconclusive in this artifact set because the screenshot completed before any visible overlay was confirmed.

## Assessment

- Functional confidence: PASS
- UX confidence upgrade: PASS
- Blocking issues found: none
- Non-blocking limitation: screenshot timing around the final transition and `Escape` capture is not frame-perfect, so the deterministic escape and modal tests remain the authoritative blocker gate for that behavior.

## Relationship To Alpha Signoff

This walkthrough is a post-alpha confidence upgrade.
The binding Alpha acceptance decision still comes from:

1. the reduced-runtime gate sweep,
2. the AXT-00 scorecard in `metrics/AXT-00-scorecard-2026-05-26.md`, and
3. the AXT-08 readiness report in `docs/tech/AXT-08-ALPHA-READINESS-REPORT-2026-05-26.md`.