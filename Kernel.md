# Broken Divinity Technical Architecture and Engineering Roadmap

**Status:** Canonical technical direction. Product decisions come from [GDD.md](GDD.md); unresolved conflicts are tracked in [docs/authority/DECISIONS-TO-LOCK.md](docs/authority/DECISIONS-TO-LOCK.md).

## Reuse-First Bevy ECS + Bevy-Ratatui ASCII Game Kernel

This document owns technical architecture and engineering milestone definitions. Detailed task steps belong in [Kernel-direction.md](Kernel-direction.md), which is an appendix and must not introduce an independent roadmap.

In this document, existing `Phase N` headings refer to engineering milestones `ENG-N`. They must not be confused with Product MVP, Product P2, or Product P3.

## Current MVP foundation target

The immediate target is to lock down the reusable kernel and establish a functioning game shell that supports a fixed dungeon loop and basic colony mechanics. The first foundation does not require procgen, raids, events, sanity, theology-driven mechanics, final factions, or the complete overworld loop.

The implementation must reuse the current `broken-divinity/` codebase where practical and preserve existing tests as regression coverage. See [docs/authority/MIGRATION-AND-DEPRECATION.md](docs/authority/MIGRATION-AND-DEPRECATION.md).

---

# 1. Project Vision

Broken Divinity will be developed as a **terminal-first, data-driven, ECS-based tactical survival game** built on a reusable Rust game kernel. The first playable dungeon is fixed and hand-authored; procedural generation is a later expansion of the reusable kernel.

This is not a custom engine from scratch.

The project is:

```text
A reusable Rust game kernel
using Bevy ECS for simulation,
Bevy-Ratatui for runtime and terminal bridge,
Ratatui for TUI rendering,
Serde/RON for content,
and external crates for infrastructure wherever practical.
```

Broken Divinity is the flagship game, but the lower-level systems should be reusable for future projects:

```text
tactics games
roguelikes
outpost sims
party-based RPGs
VN/RPG hybrids
procedural survival games
terminal management sims
```

---

# 2. Core Goals

## Goal 1 — Reuse infrastructure, build game semantics

Use existing crates for:

```text
ECS
app loop
scheduling
terminal runtime
terminal rendering
input forwarding
serialization
content loading
pathfinding
graph algorithms
save/load candidates
config loading
OS app directories
logging
panic/error reporting
testing
```

Build custom systems for:

```text
PoolDelta rules
requirements/effects
modifiers
triggers
entity factory
blueprints/mutators
semantic ASCII visual language
LocationPlan procgen staging
view-model boundaries
Broken Divinity-specific content/rules
content validation rules
```

The project should feel custom because the rules, content, and ASCII presentation are custom — not because common infrastructure was rebuilt.

---

## Goal 2 — Foundation before content

Do not start by building the full game.

First build:

```text
runtime
input
rendering
core ECS loop
PoolDelta
requirements/effects
schedule discipline
semantic ASCII
view models
content IDs
data loading
statuses/triggers/modifiers
factory/mutators
ownership
inventory/equipment
pathfinding/visibility
procgen
config
save/load
debugging
```

Then build larger Broken Divinity content.

---

## Goal 3 — Signed deltas instead of duplicate systems

Health, AP, stress, corruption, faith, morale, and other pools use a unified signed delta model.

Core rule:

```text
positive value = increase pool
negative value = decrease pool
zero = no-op
```

Examples:

```text
Health -8 = damage
Health +5 = healing

ActionPoints -2 = spend AP
ActionPoints +1 = recover AP

Stress +3 = gain stress
Stress -2 = reduce stress

Corruption +2 = become more corrupted
Corruption -1 = cleanse corruption
```

There should not be separate damage/healing systems unless they are only semantic wrappers over the same PoolDelta pipeline.

---

## Goal 4 — Signal-driven but controlled

Signals/messages/events should be used for meaningful runtime changes.

Good signals:

```text
MoveIntent
UseAbilityIntent
PoolDeltaRequested
PoolDeltaApplied
EntityMoved
StatusApplied
ItemEquipped
LocationLoaded
SelectionChanged
LogEvent
```

Bad signals:

```text
GetCurrentHealthSignal
IsTileWalkableSignal
CalculateDistanceSignal
```

Core rule:

```text
Signals announce requests and results.
Resolvers perform actual mutation.
```

---

## Goal 5 — Semantic ASCII identity

ASCII/TUI is a first-class presentation pipeline.

Gameplay should not emit raw glyphs or raw terminal colors.

Bad:

```text
'@'
'#'
Color::Red
Color::Blue
```

Good:

```text
VisualToken::Player
VisualToken::Wall
StyleToken::Enemy
StyleToken::Corruption
```

Rendering pipeline:

```text
Gameplay state
  ↓
View model
  ↓
VisualToken
  ↓
Symbol resolver
  ↓
Render layer composer
  ↓
StyleToken / theme resolver
  ↓
RenderCell grid
  ↓
Ratatui widget
  ↓
terminal
```

---

## Goal 6 — Data-driven, but not too early

Do not force unstable models into data files.

Correct order:

```text
Rust fixture
  ↓
unit tests
  ↓
small implementation
  ↓
integration test
  ↓
refactor
  ↓
move stable shape into RON/data
  ↓
add validation
```

Data loading should begin only after the Rust shape has survived real tests.

---

## Goal 7 — Production readiness through proof

The kernel is not approved because the architecture sounds good.

It must power:

```text
1. A Broken Divinity MVP loop.
2. A small standalone roguelike prototype.
```

The roguelike prototype is the final proof that the kernel is reusable and not overfit to Broken Divinity.

---

# 3. Non-Goals

Do not build these early:

```text
custom ECS
custom terminal backend
custom app loop
custom pathfinding algorithm
custom graph library
custom serialization parser
custom config parser
custom OS directory resolver
custom save serializer before save crates are spiked
full campaign
large enemy roster
complex outpost economy
advanced procgen
large world map
full modding interface
graphical Bevy renderer
```

Do not let the project become an engine vanity project.

---

# 4. Locked Technology Direction

## Runtime stack

```text
Bevy App
  ↓
Bevy ECS
  ↓
Bevy Messages / Observers
  ↓
Bevy-Ratatui
  ↓
Ratatui
  ↓
Crossterm
  ↓
Terminal
```

Bevy-Ratatui is the default runtime bridge.

This is not a maybe.

---

## Use Bevy ECS for

```text
World
Entity
Component
Resource
Query
Commands
Schedule
SystemSet
Messages
Observers
App/plugin structure
```

Do not build a custom ECS.

---

## Use Bevy-Ratatui for

```text
terminal lifecycle
raw mode handling
terminal cleanup
Ratatui context
input forwarding
Bevy/Ratatui bridge
```

Do not hand-build a terminal runtime unless Bevy-Ratatui fails Phase 0.

---

## Use Ratatui for

```text
layouts
panels
widgets
buffers
styled text
terminal frame drawing
tables
lists
gauges
```

Build custom on top:

```text
MapWidget
RenderCellGrid
VisualToken resolver
StyleToken resolver
SymbolRegistry
ThemeRegistry
overlay system
screen bindings
view models
```

---

## Use Serde/RON for content

Default content format:

```text
RON for hand-authored content
TOML for user config if useful
JSON for tooling/debug export if useful
```

Build custom:

```text
registries
ID resolution
cross-file validation
content validator
duplicate ID detection
missing reference detection
trigger loop validation
blueprint validation
screen binding validation
```

---

## Use existing pathfinding/graph crates

Default:

```text
pathfinding crate for A*/BFS/Dijkstra-style grid pathing
petgraph for room graphs, world graphs, faction graphs, procgen graphs
```

Spike:

```text
bracket-pathfinding for roguelike FOV/Dijkstra map support
```

Do not write A*, BFS, Dijkstra, FOV, or graph storage from scratch until existing crates fail.

---

# 5. External Systems We Should Not Rebuild

## Save/load and persistence

Do not build one huge custom save system first.

Use a tiered model:

```text
settings/preferences:
  bevy-persistent or bevy_persist spike

config files:
  config crate or bevy_mod_config spike

app directories:
  directories / ProjectDirs

world/run snapshots:
  bevy_save or moonshine-save spike

intent replay:
  custom, because replay semantics are game-specific

save boundaries:
  custom, because persistent/transient/location ownership is game-specific
```

The persistence strategy should be chosen after ownership, inventory, entity identity, content IDs, and location state exist.

Settings persistence and world save/load are separate decisions.

World-save spikes should test only two candidates first:

```text
1. bevy_save
2. moonshine-save
```

Only consider custom Serde snapshots if both fail.

---

## Config and directories

Do not manually guess save/config/log paths.

Use a path crate such as:

```text
directories / ProjectDirs
```

Config should eventually include:

```text
symbol mode
theme
keybindings
save directory override
log level
debug flags
accessibility options
```

---

## Input/keybindings

Start simple:

```text
Bevy-Ratatui input
  ↓
custom terminal key mapper
  ↓
GameIntent
```

Later spike:

```text
leafwing-input-manager
```

Accept Leafwing only if it reduces complexity and cleanly integrates with terminal input.

---

## Error reporting and panic handling

Use:

```text
thiserror for domain/library errors
anyhow or color-eyre for app/tooling boundary errors
tracing for structured diagnostics
```

Choose either `anyhow` or `color-eyre` during Phase 0. Do not keep both as an unresolved decision after Phase 0.

Terminal cleanup must happen on normal exit, early exit, and panic.

---

## Localization

Do not build localization now.

Avoid making future localization impossible.

Potential later spike:

```text
bevy_fluent
```

---

# 6. Dependency Tiers

## Tier 1 — Default dependencies

```text
bevy_app
bevy_ecs
bevy_time
bevy_ratatui
ratatui
crossterm
serde
ron
serde_json
toml
tracing
tracing-subscriber
thiserror
anyhow or color-eyre
insta
```

---

## Tier 2 — Strong supporting dependencies

```text
schemars
bevy_common_assets
petgraph
pathfinding
proptest
directories
config or bevy_mod_config
```

---

## Tier 3 — Spike dependencies

```text
termgrid-core
bracket-pathfinding
leafwing-input-manager
bevy_save
moonshine-save
bevy-persistent
bevy_persist
leafwing_abilities
bevy_fluent
```

---

## Tier 4 — Reference only

```text
bevy_ascii_terminal
bracket-terminal
bracket-lib full runtime
chargrid
ruscii
ascii-canvas
ascii-forge
bevy_ratatui_camera
```

---

# 7. Architecture Guardrails

## Mutation ownership

```text
Only resolver systems mutate gameplay state.
```

Rules:

```text
UI emits intents only.
AI emits intents only.
Debug mutation is gated.
Triggers emit effects only.
Modifiers modify requests only.
Effects describe requested mutations.
Resolvers perform mutations.
```

Forbidden:

```text
UI directly changes Health.
AI directly moves entities.
Status directly edits pools.
Item directly mutates components outside the effect pipeline.
Debug inspector silently edits world state.
```

---

## Signal discipline

Every signal must have:

```text
clear owner
clear schedule stage
clear reader/resolver
trace entry
failure mode
```

Trigger chains must have:

```text
max depth
cycle detection
debug trace
clear error behavior
```

---

## UI boundary

TUI systems may:

```text
read view models
draw widgets
emit input intents
```

TUI systems may not:

```text
mutate gameplay components
apply effects
resolve combat
query arbitrary gameplay internals
bypass the intent pipeline
```

---

## Debug boundary

Debug tools are read-only by default.

Debug tools may mutate the world only through:

```text
explicit DebugIntent
explicit debug-only effect path
debug mode gate
clear trace entry
```

Debug mutation must never silently bypass the architecture.

---

## Save/load boundary

Save/load may serialize and restore state.

Save/load may not silently apply gameplay rules.

Allowed:

```text
restore validated snapshot
run explicit migration
report invalid save
```

Not allowed:

```text
quietly fix invalid gameplay state
apply combat/effects during load
invent missing content without a migration rule
```

---

## ASCII boundary

No raw glyphs or raw colors outside the ASCII/theme layer.

Allowed outside ASCII layer:

```text
VisualToken::Player
VisualToken::Enemy
StyleToken::Danger
StyleToken::Corruption
```

Not allowed outside ASCII/theme layer:

```text
'@'
'#'
Color::Red
Color::Blue
```

---

## TDD rule

For core simulation systems:

```text
write failing test
implement smallest behavior
pass test
refactor
add integration test
```

Manual smoke tests are acceptable for terminal lifecycle behavior, but not for simulation logic.

---

## Phase failure rule

If a phase fails its exit criteria:

```text
stop
write a failure note
identify the blocker
simplify scope, replace dependency, or split the phase
do not proceed by working around the failure
```

A failed phase is not skipped.

---

# 8. Initial Workspace Strategy

Do not start with a dozen crates.

Start with:

```text
bd_app
bd_core
bd_tui
bd_data
bd_test_support
```

Split later only when boundaries stabilize.

Avoid premature module explosion.

---

# 9. Phase Roadmap

---

## Phase 0 — Dependency and Runtime Compatibility Gate

### Goal

Prove the runtime stack compiles, opens, draws, receives input, and exits cleanly.

### Build

```text
Cargo workspace
pinned dependency set
minimal Bevy app
Bevy-Ratatui plugin
Ratatui frame draw
input event received
quit key
clean terminal restore
DecisionLog.md
dependency matrix
basic CI/check script
docs/decisions folder
justfile or makefile
```

### Checks

```text
cargo check
cargo test
cargo fmt --check
cargo clippy
cargo run
manual terminal open/quit
panic/early-exit cleanup check
```

### Version-lock policy

Phase 0 must produce:

```text
committed Cargo.lock
accepted Bevy version
accepted Bevy-Ratatui version
accepted Ratatui version
accepted Crossterm version
dependency compatibility matrix
```

No `*` dependency versions after Phase 0.

### Early docs

Create:

```text
docs/decisions/DecisionLog.md
docs/ARCHITECTURE_GUARDRAILS.md
docs/DEPENDENCY_MATRIX.md
docs/PHASE_EXIT_CRITERIA.md
```

### Exit criteria

```text
[ ] App compiles.
[ ] Terminal opens.
[ ] Frame draws.
[ ] Key input reaches Bevy.
[ ] Quit exits cleanly.
[ ] Terminal restores after normal exit.
[ ] Terminal restores after panic/early exit.
[ ] Dependency compatibility matrix exists.
[ ] Cargo.lock is committed.
[ ] Dependency versions are pinned.
[ ] DecisionLog.md exists.
[ ] Basic CI/check script exists.
[ ] Failure policy is documented.
[ ] anyhow vs color-eyre decision is made.
```

No gameplay work begins until this passes.

---

## Phase 1 — Minimal Terminal Slice

### Goal

Prove ECS, terminal rendering, input, logs, and basic UX.

### Build

```text
SMOKE_MAP_WIDTH = 20
SMOKE_MAP_HEIGHT = 12
player entity
walls/floor
simple key mapper
MoveIntent
Position
map panel
stat panel
help line
log panel
build/version footer
```

### Early UX

```text
help line:
  WASD/Arrows move | . wait | q quit

stat panel:
  HP
  AP

log panel:
  movement result
  blocked movement reason
```

The help line is temporary until configurable input exists.

Raw `@`, wall glyphs, and simple colors are temporary fixtures until Phase 5 replaces them with semantic rendering.

### Tests first

```text
valid MoveIntent changes Position
MoveIntent into wall is rejected
invalid input does not mutate world
blocked movement emits denial log
UI does not mutate gameplay state
```

### Exit criteria

```text
[ ] @ moves.
[ ] walls block movement.
[ ] help line exists.
[ ] stat panel exists.
[ ] log panel exists.
[ ] build/version footer exists.
[ ] first ASCII snapshot test exists.
[ ] temporary glyph/color fixture usage is documented.
```

---

## Phase 2 — PoolDelta Core

### Goal

Build the signed pool mutation pipeline.

### Build

```text
PoolKind
Pool
PoolDeltaRequested
PoolDeltaApplied
PoolDeltaResolver
PoolThresholdRule
```

### Rule

```text
All pool mutation goes through PoolDelta.
```

### Tests first

```text
Health -N damages
Health +N heals
AP -N spends
AP +N restores
zero delta no-ops
clamping works
fatal Health emits defeat result
```

### Exit criteria

```text
[ ] Health/AP use same pipeline.
[ ] No separate DamageSystem/HealingSystem exists.
[ ] All pool mutation goes through PoolDelta.
[ ] Pool changes appear in trace/log.
```

---

## Phase 3 — Requirements, Effects, and Actions

### Goal

Actions become requirement/effect bundles.

### Build

```text
Requirement
Effect
ActionDefinition
TargetingRule
CostDefinition
ActionResolver
EffectResolver
denial reasons
```

### Rules

```text
requirements inspect only
effects request only
costs compile to PoolDelta
resolvers mutate only
```

### Initial actions

```text
Move
Wait
Attack
Guard
```

### Tests first

```text
move denied without AP
move denied into wall
wait restores AP
attack denied out of range
attack emits Health PoolDelta
guard applies status placeholder
denial reason is displayed
```

### Exit criteria

```text
[ ] Movement is an action.
[ ] Wait is an action.
[ ] Attack is an action.
[ ] Costs are not hardcoded in systems.
[ ] UI can show denial reasons.
```

---

## Phase 4 — Schedule and Signal Discipline

### Goal

Prevent event spaghetti before complex systems are added.

### Build schedule stages

```text
Input
IntentCollection
Validation
CostResolution
EffectEmission
ModifierApplication
Mutation
ResultEmission
Presentation
Render
```

### Build

```text
SignalTrace
schedule labels
resolver ownership rules
trigger-depth guard
cycle detection placeholder
trace snapshot test
decision on Bevy messages vs observers per signal class
```

### Tests first

```text
mutation only occurs in Mutation stage
effect does not directly mutate component
invalid UI intent is ignored
trace order is stable
trigger depth cap works
```

### Exit criteria

```text
[ ] Every signal has an owning stage.
[ ] Trace explains input -> result.
[ ] Recursive signal protection exists.
[ ] Bevy messages vs observers decision is documented.
```

---

## Phase 5 — Semantic ASCII Renderer V1

### Goal

Replace raw glyph rendering with semantic visual tokens.

### Build

```text
VisualToken
StyleToken
SymbolDef
ThemeDef
RenderLayer
RenderCell
RenderCellGrid
MapViewModel
MapWidget
```

### Keep out of scope

```text
dirty-cell optimization
advanced camera
animations
multiple full themes
debug heatmaps
```

### Optional spike

```text
termgrid-core
```

Accept only if it clearly simplifies render cells, draw ops, glyph width handling, or damage tracking without fighting Ratatui.

### Snapshot review policy

Snapshot updates must be reviewed intentionally.

Do not auto-accept changed snapshots without checking whether the visual change is expected.

### Tests first

```text
player renders over floor
wall renders correctly
enemy renders over item
selection preserves base glyph
missing visual token fails validation
missing style token fails validation
```

### Exit criteria

```text
[ ] No raw glyphs in gameplay code.
[ ] No raw colors outside theme layer.
[ ] Snapshot covers a small map.
[ ] Symbol/theme system is working.
[ ] Snapshot review policy is documented.
```

---

## Phase 6 — View Models V1

### Goal

Create a hard boundary between ECS gameplay state and TUI rendering.

### Build

```text
MapViewModel
ActorPanelViewModel
ActionListViewModel
LogViewModel
StatsViewModel
```

### Tests first

```text
view model contains visible player
view model contains current HP/AP
view model contains available actions
view model contains denial messages
TUI reads view models only
```

### Exit criteria

```text
[ ] UI does not query arbitrary gameplay internals.
[ ] UI draws from view models.
[ ] UI emits intents only.
```

---

## Phase 7 — Content IDs and Registry Core

### Goal

Define stable IDs before expanding data.

### Build

```text
ContentId
RegistryKey
ID namespace policy
duplicate ID validation
missing reference validation
basic Registry<T>
```

### ID pattern

```text
ability.smite
status.poisoned
item.rusted_spear
blueprint.human_militia
tile.ruined_wall
theme.bd_default
screen.combat_default
```

### Tests first

```text
valid ID parses
invalid ID fails
duplicate ID fails
missing reference fails
registry lookup works
```

### Exit criteria

```text
[ ] Content IDs are stable.
[ ] No ad hoc string IDs scattered through systems.
[ ] Registry errors are readable.
```

---

## Phase 8 — Data Loading V1

### Goal

Move only stable fixtures into RON.

### Build

```text
serde derives
RON loading
SymbolRegistry
ThemeRegistry
ActionRegistry only if stable
ContentError
basic content validator
```

### Use external

```text
serde
ron
bevy_common_assets if asset flow is active
schemars after content types stabilize
```

### Data-loading order

Move data in this order:

```text
1. symbols
2. themes
3. actions only if action schema has stabilized
```

Do not force action data into RON if requirements/effects are still changing.

### Tests first

```text
loads valid RON symbol
loads valid RON theme
rejects duplicate ID
rejects unknown style token
rejects unknown visual token
loads valid RON action if action schema is accepted
```

### Schema generation

Spike `schemars` only after the first loaded content types are stable.

### Exit criteria

```text
[ ] Symbols load from data.
[ ] Theme loads from data.
[ ] Move/Wait/Attack are registry-backed only if schema is stable.
[ ] Errors are readable.
[ ] Schemars timing decision is documented.
```

---

## Phase 9 — Statuses, Triggers, and Modifiers V1

### Goal

Add nuance without bespoke systems.

### Build

```text
Status
StatusInstance
Trigger
TriggeredEffect
Modifier
ModifierContext
TriggerContext
duration ticking
stack policy
modifier ordering
modifier trace entries
```

### Initial statuses

```text
Poisoned
Regeneration
Guarded
Blessed
Broken Choir Static
```

### Tests first

```text
poison triggers Health -2 on turn start
regen triggers Health +3 on turn start
armor modifies negative Health delta
blessed modifies positive Health delta
Broken Choir Static inverts tagged Divine healing
status expires after duration
trigger loop is detected or capped
```

### Exit criteria

```text
[ ] Triggers emit effects only.
[ ] Modifiers modify requests only.
[ ] Trigger loops cannot hang game.
[ ] Modifier order is traceable.
```

---

## Phase 10 — Entity Factory V1

### Goal

Spawn entities through blueprints and mutators.

### Build

```text
EntityBlueprint
SpawnRequest
SpawnContext
Mutator
FactoryResolver
BlueprintRegistry
SpawnValidationReport
```

### Initial blueprints

```text
player
human_militia
basic_enemy
rusted_spear
simple_trap
exit_tile
```

### Tests first

```text
spawn player blueprint
spawn enemy blueprint
spawn item blueprint
spawn trap blueprint
apply wounded mutator
apply elite mutator
apply faction mutator
invalid blueprint rejected
```

### Exit criteria

```text
[ ] Factory spawns actors/items/traps/exits.
[ ] Mutators are composable.
[ ] No enemy-specific spawn system exists.
```

---

## Phase 11 — Relationships and Ownership

### Goal

Define ownership before inventory, summons, save/load, and transitions depend on it.

### Build

```text
OwnedBy
ContainedIn
EquippedBy
SummonedBy
LocationOwned
FactionMember
```

### Tests first

```text
summon has owner
item can be contained
item can be equipped
location-owned entity can be queried
transient entity can be identified
```

### Exit criteria

```text
[ ] Ownership model exists.
[ ] Equipment/inventory can use relationships.
[ ] Save/load can identify ownership.
```

---

## Phase 12 — Inventory, Equipment, and Containers

### Goal

Build the generic item/container layer before save/load or roguelike prototype.

### Build

```text
Container
ItemEntity policy
EquipmentSlot
PickupIntent
DropIntent
EquipIntent
UnequipIntent
UseItemIntent
ContainerViewModel
```

### Tests first

```text
pickup transfers item
drop places item on map
equip moves item to slot
invalid slot is rejected
use item emits effects
container persists as data
```

### Exit criteria

```text
[ ] Inventory works.
[ ] Equipment works.
[ ] Item use goes through effects.
[ ] UI can show container view model.
```

---

## Phase 13 — Pathfinding and Visibility Spike

### Goal

Pick pathing/FOV crates through adapters.

### Compare

```text
pathfinding
bracket-pathfinding
```

### Tests first

```text
path avoids walls
movement range respects AP
occupied blocking tiles work
unreachable target returns no path
visibility hides unseen actor
remembered tile renders muted
```

### Build

```text
TileMap adapter
OccupancyMap
PathMap
VisibilityMap
MovementRange
LineOfSight/FOV adapter
```

### Exit criteria

```text
[ ] Pathfinding decision recorded.
[ ] Pathfinding is behind adapter.
[ ] Movement range overlay works.
[ ] Visibility states work.
```

---

## Phase 14 — Procedural Location V1

### Goal

Generate valid small locations through staged plans, not direct world mutation.

### Build

```text
LocationTemplate
LocationPlan
RoomGraph
TilePainter
SpawnZone
ExitPoint
LocationMutator
LocationValidator
```

### Tests first

```text
same seed generates same map
all rooms reachable
exit reachable
spawn zones valid
no actor spawns inside wall
required entrance exists
```

### Exit criteria

```text
[ ] Procgen creates plan first.
[ ] Plan validates before spawning.
[ ] Seed reproduces bugs.
```

---

## Phase 15 — Data-Driven TUI Screens

### Goal

Move from hardcoded panels to reusable screen definitions.

### Phase 15A — One schema-driven combat screen

Build:

```text
ScreenDefinition
PanelDefinition
WidgetBinding
WidgetRegistry
ScreenState
ViewModelRegistry
combat screen definition
```

Tests first:

```text
combat screen resolves bindings
missing widget ID fails validation
missing view model binding fails validation
TUI cannot mutate gameplay directly
```

Exit criteria:

```text
[ ] Combat screen is schema-driven.
[ ] Widgets consume view models only.
```

### Phase 15B — Second screen proves reuse

Build:

```text
inventory screen definition
second screen validation path
screen switching
```

Tests first:

```text
inventory screen resolves bindings
screen switching preserves gameplay state
missing inventory view model fails validation
```

Exit criteria:

```text
[ ] Inventory screen is schema-driven.
[ ] Screen system proves reuse across at least two screens.
```

---

## Phase 16 — Config, Preferences, and App Directories

### Goal

Avoid custom config/path logic before production.

### Build

```text
AppConfig
KeyBindingConfig
ThemeConfig
SavePathConfig
LogConfig
ProjectDirs integration
config loading
config validation
default config behavior
```

### External candidates

```text
directories
config
bevy_mod_config
bevy-persistent
bevy_persist
```

### Tests first

```text
default config loads
missing config uses defaults or creates default config
bad config fails with readable error
config directory resolves
save directory resolves
theme setting resolves
keybinding setting resolves
```

### Help-line rule

By the end of this phase, the help line should derive from the same input binding source as the input mapper.

### Exit criteria

```text
[ ] No hand-rolled OS path guessing.
[ ] Config is user-editable.
[ ] Settings can persist.
[ ] Help line and input mapper use the same binding source.
```

---

## Phase 17 — Save / Load / Replay Spikes

### Goal

Choose persistence strategy after identity, inventory, ownership, and location state exist.

### Separate decisions

Settings persistence and world save/load are separate decisions.

### Compare for world snapshots

Test first:

```text
bevy_save
moonshine-save
```

Only consider custom Serde snapshots if both fail.

### Build

```text
PersistentEntity
TransientEntity
LocationSnapshot
RunSnapshot
IntentReplayLog
SaveVersion
ContentVersion
```

### Tests first

```text
player persists
position persists
pools persist
inventory persists
equipment persists
transient summon excluded
location seed persists
save version recorded
content version mismatch handled
intent replay deterministic
```

### Fixed-seed replay

Add a fixed-seed intent replay script before full production save/load.

This helps test deterministic action resolution even before save/load is mature.

### Decision rule

```text
Use external crate for serialization/storage if it fits.
Keep save boundaries and replay custom.
Reject any save crate that forces bad entity identity or persistence rules.
```

### Exit criteria

```text
[ ] Save strategy accepted.
[ ] Settings persistence strategy accepted.
[ ] Small world roundtrip works.
[ ] Replay reproduces short sequence.
[ ] Save/load does not apply gameplay rules during load.
```

---

## Phase 18 — Broken Divinity Tactical MVP

### Goal

Build the first actual playable BD tactical loop.

### Content targets

These are acceptance targets, not hardcoded caps.

```text
1 player archetype
2 enemy archetypes
1 summon/ally archetype
5 abilities
5 statuses
5 items
3 tile types
1 generated location template
combat screen
inventory screen
debug screen
```

### Loop

```text
enter generated ruin
move
fight
use abilities/items
collect loot
reach exit
return to placeholder outpost
save/load run
```

### Exit criteria

```text
[ ] Playable 10-minute tactical loop.
[ ] Player can win.
[ ] Player can die.
[ ] Save/load works.
[ ] Debug trace explains major actions.
```

---

## Phase 19 — Outpost, Travel, and Transitions V1

### Goal

Connect tactical play to broader BD structure.

### Build

```text
outpost state
travel nodes
location transitions
resource pools
basic production timers
party carryover
persistent location memory
```

### Tests first

```text
leaving location preserves player
returning to outpost works
travel advances time
outpost resources change through pool-like system
transient combat entities do not leak into outpost
```

### Exit criteria

```text
[ ] Tactical and outpost modes connect.
[ ] Travel/time works simply.
[ ] No major state leaks between modes.
```

---

## Phase 20 — UX, Debugging, and Tooling Hardening

### Goal

Make the game usable and debuggable.

### Build

```text
input help
action denial messages
debug overlay
entity inspector
event trace viewer
content validation CLI
procgen preview command
symbol/theme preview command
crash-safe terminal recovery
```

### External candidates

```text
tracing
color-eyre
insta
proptest
```

### Exit criteria

```text
[ ] Invalid action shows reason.
[ ] Missing content gives readable error.
[ ] Validator catches bad references.
[ ] Terminal recovers after panic path.
[ ] Debug mutation is gated.
```

---

## Phase 21 — Performance and Stability

### Goal

Measure before optimizing.

### Measure

```text
frame time
input latency
render cost
pathfinding cost
procgen cost
save/load time
memory growth
event queue growth
trigger chain depth
```

### Optimize only if needed

```text
dirty-cell rendering
cached view models
pathfinding cache
visibility cache
render allocation reduction
registry indexing
```

### Tests

```text
stability simulation does not leak entities
large event chain terminates
seed batch does not panic
render snapshot remains stable
save/load stress test passes
```

### Exit criteria

```text
[ ] No runaway event queues.
[ ] No uncontrolled entity growth.
[ ] Seed batch does not panic.
[ ] Render remains responsive.
[ ] Optimizations are measurement-driven.
```

---

## Phase 22 — Packaging and Release Candidate

### Goal

Create a build that runs outside the dev environment.

### Build

```text
release profile
content folder layout
config file
default keybindings
default theme
save directory
logs directory
README
troubleshooting guide
version display
scripted release smoke test
```

### Checks

```text
fresh checkout builds
release binary runs
content files package correctly
config loads
logs write
save folder is created
terminal exits cleanly
```

### Exit criteria

```text
[ ] Fresh checkout builds.
[ ] Release binary runs.
[ ] Content files package correctly.
[ ] Config loads.
[ ] Logs write.
[ ] Save folder is created.
[ ] Terminal exits cleanly.
[ ] Scripted release smoke test exists.
```

---

## Phase 23 — Standalone Roguelike Prototype Approval

### Goal

Prove the kernel is reusable without building a second full game.

This is a validation harness, not a second product.

### Reduced scope

These are acceptance targets, not hardcoded caps.

```text
1 player
2 enemy types
1 special ability
2 statuses
3–5 items
1 weapon type
1 armor/relic slot
4 tile types
1 dungeon template
2 floors
1 final encounter
basic inventory
basic equipment
basic loot
save/load
death/win state
summary screen
```

### Must use same systems

```text
PoolDelta
requirements/effects
statuses/triggers
modifiers
entity factory
procgen
pathfinding
visibility
semantic ASCII
data-driven UI
save/load
content validation
debug trace
```

### Must not cheat

```text
no bypassing intent/effect pipeline
no special combat system
no raw glyph rendering outside ASCII layer
no spawning outside factory
no custom one-off procgen
no disabled save/load
no prototype-specific architecture
```

### No-polish rule

Do not add:

```text
unique UI polish
special balance pass
extra content beyond acceptance scope
prototype-only architecture
```

### Required deterministic test

```text
fixed-seed deterministic prototype run
fixed intent script
expected win/death/summary outcome
```

### Exit criteria

```text
[ ] Prototype playable from start to win/death.
[ ] No prototype-only hacks.
[ ] Save/load works.
[ ] Generated floors are reachable.
[ ] Content validator passes.
[ ] Logs/traces explain bugs.
[ ] Fixed-seed prototype run is reproducible.
```

---

## Phase 24 — Production-Ready Gate

### Goal

Approve the kernel and Broken Divinity MVP for larger production.

### Required gates

```text
[ ] All tests pass.
[ ] Content validator passes.
[ ] Terminal cleanup is reliable.
[ ] Save/load roundtrip passes.
[ ] Broken Divinity MVP works.
[ ] Outpost/travel transition works.
[ ] Standalone roguelike prototype works.
[ ] No prototype-only hacks required.
[ ] Dependency versions pinned.
[ ] Release build runs from clean folder.
[ ] Known issues documented.
```

---

# 10. Spike Rules

Every spike must be timeboxed and recorded in `DecisionLog.md`.

Each spike must include:

```text
problem
crate(s) tested
accept criteria
reject criteria
fallback
decision
reason
date
```

Current spikes:

```text
termgrid-core:
  accept only if it simplifies render grid enough to justify dependency

bracket-pathfinding:
  accept only if FOV/Dijkstra support beats pathfinding crate adapter

leafwing-input-manager:
  accept only if terminal input maps cleanly with less code

bevy_save:
  accept only if save boundaries remain under our control

moonshine-save:
  accept only if it handles Bevy save/load better than bevy_save for our world model

bevy-persistent / bevy_persist:
  accept for settings/resources if simple and compatible

config / bevy_mod_config:
  accept if it reduces config loading/validation boilerplate

color-eyre:
  accept if panic/error reports integrate cleanly with terminal cleanup

bevy_fluent:
  defer until localization is a real product goal
```

---

# 11. Extension Policy

Early systems may use enums for speed:

```text
Effect
Requirement
Modifier
Trigger
VisualToken
StyleToken
```

This is acceptable for the MVP.

When any enum or match block becomes difficult to extend, move toward:

```text
registered handlers
plugin-style extension
data-backed routing
small handler modules
```

Do not prematurely abstract.

Do not let one giant match block become permanent.

---

# 12. Resolver / Validator SRP Policy

Large facades are allowed only as orchestration layers.

They must not become god objects.

Examples:

```text
EffectResolver:
  may route effects
  should delegate to small effect handlers

FactoryResolver:
  may orchestrate spawning
  should delegate component construction/mutator logic

ContentValidator:
  may orchestrate validation
  should delegate:
    ID validation
    reference validation
    schema validation
    trigger-cycle validation
    blueprint validation
    screen-binding validation

MapWidget:
  may draw the map
  must not calculate gameplay state

Save layer:
  may serialize/restore state
  must not decide game rules
```

---

# 13. Validation Checklist

## Architecture

```text
[ ] Only resolvers mutate gameplay state.
[ ] UI emits intents only.
[ ] AI emits intents only.
[ ] Debug mutation is gated.
[ ] Effects request mutation only.
[ ] Triggers emit effects only.
[ ] Modifiers modify requests only.
[ ] Signal stages documented.
[ ] Trigger recursion capped.
[ ] Save/load does not apply gameplay rules during load.
```

---

## Reuse

```text
[ ] No custom ECS.
[ ] No custom terminal backend.
[ ] No custom pathfinding algorithm.
[ ] No custom OS directory resolver.
[ ] No custom config parser.
[ ] No custom panic reporter unless external crate fails.
[ ] No custom save serializer before save crates are spiked.
```

---

## Data-driven

```text
[ ] Stable ContentId exists.
[ ] ID namespaces documented.
[ ] Duplicate IDs fail validation.
[ ] Missing references fail validation.
[ ] Raw glyphs isolated to symbol data.
[ ] Raw colors isolated to theme data.
[ ] Balance values move to content data once stable.
[ ] Help text derives from input bindings once config exists.
```

---

## UX

```text
[ ] Help line exists early.
[ ] HP/AP visible early.
[ ] Invalid actions show reasons.
[ ] Log panel explains results.
[ ] Quit behavior is clear.
[ ] Debug trace can be toggled.
[ ] Content errors are readable.
[ ] Build/version footer exists.
```

---

## Persistence

```text
[ ] Config path resolved through external path crate or accepted persistence crate.
[ ] Settings persistence decided.
[ ] World save strategy decided.
[ ] Replay log remains custom.
[ ] Transient entities excluded.
[ ] Inventory/equipment persists.
[ ] Save version recorded.
[ ] Content version behavior defined.
[ ] Fixed-seed replay works.
```

---

## Final approval

```text
[ ] Broken Divinity tactical MVP works.
[ ] Outpost/travel transition works.
[ ] Standalone roguelike prototype works.
[ ] Prototype uses same kernel.
[ ] Release build runs.
[ ] Known issues documented.
```

---

# 14. Final Implementation Rule

The development rhythm is:

```text
prove the stack
pin dependencies
build tiny playable slice
add one system at a time
test before implementation
move stable shapes into data
reuse external crates where possible
delay save/procgen/UI complexity until prerequisites exist
validate reuse with a small roguelike prototype
then declare production readiness
```

Do not compress the phases.

Do not build the whole kernel at once.

Do not write custom infrastructure until the relevant crate has failed a spike.

Do not proceed past failed exit criteria.

The final product should be a stable reusable game kernel, not a pile of clever custom systems recreating existing libraries.
