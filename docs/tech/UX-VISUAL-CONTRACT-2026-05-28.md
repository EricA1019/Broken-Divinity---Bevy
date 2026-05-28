# UX Visual Contract v0 (Phase 0)

Date: 2026-05-28
Branch: feat/ui-visual-contract-animations
Status: Active exploratory baseline (not production-final)

## Goal

Lock one coherent visual language before broad reskin work:
- Crimson/Gold/Black palette
- Symbol grammar
- Typography and spacing hierarchy
- Motion behavior constraints
- Widget-level animation experiments that remain readable under tactical pressure

## External Reference Snapshot (used for direction)

Sources reviewed:
- Bay 12 Dwarf Fortress dev logs and UI notes: https://www.bay12games.com/dwarves/
- Caves of Qud site and screenshots: https://www.cavesofqud.com/
- Cataclysm: DDA site and docs links: https://cataclysmdda.org/
- Grid Sage Games/Cogmind UI upscaling and design posts: https://www.gridsagegames.com/blog/
- Brogue summary and references: https://en.wikipedia.org/wiki/Brogue_(video_game)

Observed recurring strengths across these ecosystems:
- Strong information hierarchy: immediate tactical data, then context, then deep detail.
- High-contrast foreground/background with restrained accent colors.
- Dense but stable panel geography (players can build muscle memory).
- Log/event stream as a first-class element.
- Motion used sparingly and purposefully (attention signaling, not decoration).

## Contract: Palette Tokens

Primary semantic colors:
- Title Gold: rgb(228, 190, 72)
- Subtitle Gold: rgb(178, 146, 96)
- Accent Crimson: rgb(186, 34, 48)
- Warning Amber: rgb(220, 160, 50)
- Danger Crimson-High: rgb(232, 54, 68)
- Success Green: rgb(116, 188, 112)
- Info Warm Tan: rgb(192, 152, 110)
- Panel Background Near-Black: rgb(8, 4, 4)

Usage rules:
- Gold carries identity and structural headers.
- Crimson carries action urgency and primary highlights.
- Amber carries caution only (never success).
- Danger is reserved for critical state or destructive action.
- Success green is narrow-use to avoid visual noise.

## Contract: Symbol Grammar

Primary symbol set:
- Corners and borders: ╔ ═ ╗ ║ ╚ ╝
- Divider: │
- Primary action bullet: ▶
- Warning marker: !
- Success marker: +

ASCII-safe fallback set (required for low-glyph contexts):
- Corners and borders: + - + | + +
- Divider: |
- Primary action bullet: >
- Warning marker: !
- Success marker: +

Rules:
- Decorative glyphs are allowed only in section boundaries and headers.
- Core instruction copy must remain plain and parseable without decorations.
- Any symbol used for meaning must have text redundancy (example: "! Supply Risk").

## Contract: Type and Spacing

Type tiers:
- Heading: 20
- Body: 16
- Small: 12
- Monospace-first rendering in prototype surfaces

Spacing tier:
- Baseline multiplier: 1.0
- Use whole-step spacing blocks (6, 8, 10, 12, 16, 24, 32)

Rules:
- Never compress primary CTA rows below body size.
- Keep hint rows at small size but ensure contrast to background.
- Keep panel rhythm consistent across all screens.

## Contract: Panel Grammar

Every major screen should be legible as:
- Header strip
- Core content block(s)
- Action row
- Hint row

Do not invert this hierarchy between screens unless the screen is modal-critical.

## Motion Profiles for Widget Experiments

Profiles now enabled in prototype:
- Subtle
- Pulse
- Scanline

Current animated widget lab includes:
- Urgency chip (color modulation)
- Scanner progress bar (moving progress)
- Log stream spinner (frame cadence)

Motion constraints:
- Max one high-salience animated element per horizontal group.
- Keep animations periodic and predictable, avoid stochastic jitter.
- Preserve readability at all times; motion cannot obscure text semantics.

## Prototype Control Mapping

Current controls in prototype binary:
- M/C/D for screen switching
- 1/2/3 for motion profile switching
- Esc to quit

## Implementation Anchor

Centralized contract module:
- src/ui/ux_style_contract.rs

Prototype consumer:
- src/ui/ux_prototypes.rs

## Branch Policy

This contract and animation experiments stay on branch:
- feat/ui-visual-contract-animations

No merge to main until:
- Overworld and Dialogue mockups are added under this contract
- Full production UI wiring matrix is completed
- Contract and behavior gates are approved
