---
name: install-external-cli
description: Installing or updating a third-party CLI in the workspace and wiring any platform-specific assistant integration
triggers:
  - "install tool"
  - "third-party cli"
  - "graphify"
  - "workspace tool"
edges:
  - target: "context/setup.md"
    condition: "when the task is about installing, running, or verifying tooling in the local environment"
  - target: "context/conventions.md"
    condition: "when you need the verify checklist before updating workspace metadata"
last_updated: 2026-04-10
---

# Install External CLI

## Context
- Load `context/architecture.md` first for session bootstrap, then `context/setup.md` for local-environment expectations.
- Check whether the tool's repository is already present in the workspace before cloning or downloading anything.
- Read the tool's own install docs (`README`, `pyproject.toml`, package entrypoints) so you install the right distribution and post-install hook for the active assistant.

## Steps
1. Inspect the workspace for an existing checkout, dirty state, and local install status (`which`, import path, installed version, generated config files).
2. Compare the installed package version to the checked-out repo version so upgrades are deliberate instead of blind reinstalls.
3. Install from the local checkout when the repo is already in the workspace and the user asked to install that repo here; prefer a user-scoped editable install for Python CLIs unless the project docs say otherwise.
4. Run the platform-specific integration step after the package install (for example, `graphify copilot install`) so the CLI is actually usable from the active assistant.
5. Verify the final state with the real entrypoints the user will use: package version, binary resolution, and any generated skill/config file.
6. If the task was significant, update `.mex/ROUTER.md` so future sessions know the tool is already available.

## Gotchas
- The package name and CLI command may differ (`graphifyy` on PyPI vs `graphify` on the command line).
- A stale global install can mask the workspace checkout; always compare import path and version before and after installation.
- Some assistant integrations are user-level (`~/.copilot/skills/...`) rather than project-local. Installing the package alone may not enable the command inside the assistant.
- Optional extras (`[pdf]`, `[video]`, `[all]`) can be heavy; only install them when the user asked for those capabilities.

## Verify
- [ ] The installed package version matches the intended checkout or release
- [ ] The CLI resolves from the expected interpreter/environment
- [ ] Platform-specific skill or config files were created in the expected location
- [ ] No unnecessary project files were changed beyond intentional workspace metadata updates

## Debug
- If the wrong version still resolves, inspect `which <tool>`, `python3 -m pip show <package>`, and the import path from `python3 -c`.
- If the assistant command is unavailable, inspect the generated skill/config directory and rerun the platform-specific install subcommand.
- If editable install fails, retry with a non-editable user install from the same checkout to separate packaging issues from path-link issues.

## Update Scaffold
- [ ] Update `.mex/ROUTER.md` "Current Project State" if what's working/not built has changed
- [ ] Update any `.mex/context/` files that are now out of date
- [x] If this is a new task type without a pattern, create one in `.mex/patterns/` and add to `INDEX.md`
