# QA Log Profiles

## Purpose
Provide two explicit launch profiles for QA workflows:
- Standard QA: low-noise logs with gameplay-relevant signal.
- Deep diagnostics: higher verbosity for investigation sessions.

## Profiles

### Standard QA Profile
Command:
- cargo run -- --qa-standard

Behavior:
- Startup log filter: warn,broken_divinity=info
- Intended use: routine playtests and UX validation.

### Deep Diagnostics Profile
Command:
- cargo run -- --qa-deep-diagnostics

Behavior:
- Startup log filter: info,broken_divinity=debug
- Intended use: debugging and issue reproduction sessions.

## Headless Smoke Commands
Use these in CI-like smoke checks where the process is expected to keep running:
- timeout 8s cargo run --quiet -- --headless --qa-standard
- timeout 8s cargo run --quiet -- --headless --qa-deep-diagnostics

Expected result:
- Exit code 124 from timeout, indicating successful startup and sustained run loop.

## Notes
- If a custom RUST_LOG is set externally, behavior should be reviewed as part of QA setup.
- Use the standard profile by default to keep failure signals visible.
