# Pattern Index

Lookup table for all pattern files in this directory. Check here before starting any task — if a pattern exists, follow it.

<!-- This file is populated during setup (Pass 2) and updated whenever patterns are added.
     Each row maps a pattern file (or section) to its trigger — when should the agent load it?

     Format — simple (one task per file):
     | [filename.md](filename.md) | One-line description of when to use this pattern |

     Format — anchored (multi-section file, one row per task):
     | [filename.md#task-first-task](filename.md#task-first-task) | When doing the first task |
     | [filename.md#task-second-task](filename.md#task-second-task) | When doing the second task |

     Example (from a Flask API project):
     | [add-api-client.md](add-api-client.md) | Adding a new external service integration |
     | [debug-pipeline.md](debug-pipeline.md) | Diagnosing failures in the request pipeline |
     | [crud-operations.md#task-add-endpoint](crud-operations.md#task-add-endpoint) | Adding a new API route with validation |
     | [crud-operations.md#task-add-model](crud-operations.md#task-add-model) | Adding a new database model |

     Keep this table sorted alphabetically. One row per task (not per file).
     If you create a new pattern, add it here. If you delete one, remove it. -->

| Pattern | Use when |
|---------|----------|
| [add-component-resource.md](add-component-resource.md) | Adding a new ECS component or resource to the project |
| [add-egui-panel.md](add-egui-panel.md) | Adding a new egui panel, window, modal, or menu |
| [add-system.md](add-system.md) | Adding a new Bevy ECS system to the project |
| [artifact-hygiene.md](artifact-hygiene.md) | Cleaning up oversized Cargo build outputs or adding repeatable target-pruning hygiene |
| [audit-copilot-skills.md](audit-copilot-skills.md) | Running a maintenance audit on Copilot skills, instructions, and MCP config |
| [examine-codebase.md](examine-codebase.md) | Running a full repo examination, architecture review, or codebase health audit across Broken Divinity |
| [expand-save-schema.md](expand-save-schema.md) | Expanding the save/load schema with nested states and backward-compatible legacy-field loading |
| [gate-real-time-sim.md](gate-real-time-sim.md) | Adding or repairing explicit real-time pacing for colony/overworld simulation, shared timers, and `GameTime` advancement |
| [install-external-cli.md](install-external-cli.md) | Installing or updating a third-party CLI in the workspace and wiring assistant-specific setup |
| [rebalance-early-combat.md](rebalance-early-combat.md) | Softening a lethal early combat loop by reducing first-floor pressure, binding attacks to real weapon/enemy profiles, and validating with tests plus BRP smoke |
| [split-home-away-raids.md](split-home-away-raids.md) | Separating at-home raid handling from away auto-resolution while preserving colony survivors/stations across Colony/Overworld handoffs |
| [stop-post-death-actions.md](stop-post-death-actions.md) | Ensuring fatal player hits immediately queue GameOver and halt the rest of the enemy turn without duplicate death handling |
| [sync-scope-docs.md](sync-scope-docs.md) | Reconciling roadmap, detailed gameplay docs, and dev-plan slices after design changes or contradiction reviews |
| [wire-gabriel-intro-dungeon.md](wire-gabriel-intro-dungeon.md) | Tagging the first dungeon, carrying node-specific dungeon context, and staging Gabriel's scripted floor-2 intro plus ghost companion join |
| [wire-perk-and-sanity-loop.md](wire-perk-and-sanity-loop.md) | Wiring combat XP, perk unlock popups, passive perk effects, and sanity threshold behavior into the dungeon loop |
| [wire-save-restoration.md](wire-save-restoration.md) | Wiring Load Game, state-entry restoration, Save & Quit, and runtime player bridging across app states |
