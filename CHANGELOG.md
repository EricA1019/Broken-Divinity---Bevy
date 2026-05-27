# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- Alpha acceptance artifacts for recovery, scorecarding, signoff, and live-smoke evidence packaging — `docs/tech/`, `metrics/`
- A front-door desktop live-smoke artifact set with captured window states for the reduced runtime — `docs/tech/playtest-artifacts/2026-05-27-live-smoke/`

### Changed
- The active runtime now uses a reduced composed path with a live menu and deterministic Menu -> Colony -> Overworld -> Dungeon loop — `src/runtime_app.rs`
- The crate export surface now exposes the compatibility modules required by the reduced runtime and active Alpha contract tests — `src/lib.rs`

### Fixed
- Restored a visible playable startup path after the earlier blank-runtime failure — `src/main.rs`, `src/runtime_app.rs`
- Recovered missing compatibility surfaces for save/runtime state, readability, help/modal policy, recap behavior, and deterministic overworld weather — `src/core/`, `src/ui/`, `src/game/overworld/weather.rs`