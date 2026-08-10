# Broken Divinity Technical Execution Appendix

**Status:** Detailed implementation reference subordinate to [Kernel.md](Kernel.md). Do not treat this file as an independent source of truth. If it conflicts with `Kernel.md`, stop and resolve the conflict there before implementing.

## Phase-by-Phase Implementation Breakdown

The phase numbers in this file are engineering milestones. They are not the Product MVP, Product P2, or Product P3 labels in `GDD.md`.

The current foundation target is narrower than the eventual game: stabilize the shell and kernel, then connect the minimum dungeon and colony mechanics using the existing implementation where practical. The first dungeon is fixed and hand-authored; procgen is deferred. Raids, colony events, sanity, and theology-driven mechanics are also deferred.

## Foundation scope override

This appendix contains detailed engineering direction, including future
procedural-location work and broader tactical MVP targets. For current
implementation, [docs/archive/FOUNDATION-STABILIZATION-PLAN.md](docs/archive/FOUNDATION-STABILIZATION-PLAN.md)
is the task authority. The completed
[Foundation Recovery Plan](docs/archive/FOUNDATION-RECOVERY-PLAN.md) remains prior
evidence. Any phase text that assumes procedural generation,
raids, events, sanity, or the complete overworld loop is future work unless the
active plan explicitly brings it in.

---

# 0. Cross-Cutting Execution Rules

## Repository conventions

Use a workspace layout that starts small:

```text
broken_divinity/
  Cargo.toml
  Cargo.lock
  justfile
  README.md

  crates/
    bd_app/
    bd_core/
    bd_tui/
    bd_data/
    bd_test_support/

  content/
    symbols/
    themes/
    actions/
    blueprints/
    statuses/
    items/
    locations/
    screens/

  config/
    default.toml

  docs/
    ARCHITECTURE_GUARDRAILS.md
    DEPENDENCY_MATRIX.md
    PHASE_EXIT_CRITERIA.md
    decisions/
      DecisionLog.md

  tests/
    fixtures/
    snapshots/
```

Do not split `bd_core` into many crates yet. Inside `bd_core`, use modules first:

```text
bd_core/src/
  lib.rs
  app_state.rs
  ids.rs
  pools.rs
  actions.rs
  signals.rs
  schedule.rs
  statuses.rs
  modifiers.rs
  factory.rs
  relationships.rs
  inventory.rs
  spatial.rs
  procgen.rs
  save.rs
  trace.rs
```

Only split a module into a crate after its public API stabilizes.

---

## Plugin organization

Each major layer should expose one Bevy plugin:

```text
BdCorePlugin
BdTuiPlugin
BdDataPlugin
BdDebugPlugin
BdRuntimePlugin
```

Inside each plugin, register systems into named sets. Do not add loose systems without a schedule label.

Example system-set categories:

```text
BdSet::Input
BdSet::IntentCollection
BdSet::Validation
BdSet::CostResolution
BdSet::EffectEmission
BdSet::ModifierApplication
BdSet::Mutation
BdSet::ResultEmission
BdSet::Presentation
BdSet::ViewModelBuild
BdSet::Render
```

The execution order is part of the architecture and must be testable.

---

## Error conventions

Use typed errors inside crates:

```text
ContentError
ValidationError
ActionError
FactoryError
SaveError
RenderError
ConfigError
```

Use app-level reporting at the boundary:

```text
bd_app:
  anyhow or color-eyre

bd_core / bd_data / bd_tui:
  thiserror
```

Do not use stringly typed errors for systems that tests need to inspect.

---

## Test naming conventions

Use clear test names:

```text
pool_delta_negative_health_reduces_current_health
move_intent_into_wall_is_rejected
content_registry_rejects_duplicate_ids
factory_applies_wounded_mutator
inventory_pickup_moves_item_into_container
```

Each phase should add tests before implementation for core logic.

---

## Decision logging

Every spike creates a short record:

```text
docs/decisions/YYYY-MM-DD-short-name.md
```

Template:

```text
# Decision: <name>

## Problem

## Options tested

## Accept criteria

## Reject criteria

## Result

## Reason

## Follow-up work
```

---

# Phase 0 — Dependency and Runtime Compatibility Gate

## Objective

Create the workspace, prove Bevy-Ratatui can run a terminal app, lock versions, and establish project hygiene.

## Implementation sequence

### 0.1 Create workspace

Create:

```text
Cargo.toml
crates/bd_app/Cargo.toml
crates/bd_core/Cargo.toml
crates/bd_tui/Cargo.toml
crates/bd_data/Cargo.toml
crates/bd_test_support/Cargo.toml
```

Workspace members:

```text
crates/bd_app
crates/bd_core
crates/bd_tui
crates/bd_data
crates/bd_test_support
```

`bd_app` is the binary crate.

The others are library crates.

---

### 0.2 Add initial dependencies

Start only with runtime essentials:

```text
bevy_app
bevy_ecs
bevy_time
bevy_ratatui
ratatui
crossterm
tracing
tracing-subscriber
thiserror
insta
```

Choose either:

```text
anyhow
```

or:

```text
color-eyre
```

during this phase.

Do not add data, save, procgen, or pathfinding dependencies yet.

---

### 0.3 Create command runner

Add `justfile` or equivalent with:

```text
just check
just test
just fmt
just clippy
just run
just ci
```

Suggested command meanings:

```text
check:
  cargo check --workspace

test:
  cargo test --workspace

fmt:
  cargo fmt --all -- --check

clippy:
  cargo clippy --workspace --all-targets -- -D warnings

run:
  cargo run -p bd_app

ci:
  check + test + fmt + clippy
```

---

### 0.4 Minimal app startup

In `bd_app`, create the minimum runtime:

```text
main
  initialize error reporting
  initialize tracing
  create Bevy App
  add Bevy-Ratatui plugin
  add minimal draw system
  add input handling system
  run app
```

The draw system should display:

```text
Broken Divinity Runtime Test
Press q to quit
```

No game state yet.

---

### 0.5 Terminal cleanup validation

Confirm:

```text
normal quit restores terminal
panic path restores terminal
early error restores terminal
```

Add a manual panic trigger behind a debug key or feature flag for this phase only.

Document result in:

```text
docs/decisions/terminal-cleanup.md
```

---

### 0.6 Version lock

After the app works:

```text
commit Cargo.lock
replace wildcard versions
record accepted versions in DEPENDENCY_MATRIX.md
```

Dependency matrix columns:

```text
crate
version
purpose
accepted/rejected
notes
fallback
```

---

## Tests/checks

Automated:

```text
cargo check --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Manual:

```text
terminal opens
terminal draws text
q exits
terminal state is restored
panic path restores terminal
```

## Phase artifact

```text
running terminal shell
pinned dependency set
DecisionLog.md
DEPENDENCY_MATRIX.md
justfile
basic CI command
```

---

# Phase 1 — Minimal Terminal Slice

## Objective

Render a tiny map, move a player, show help/stats/logs, and prove UI does not mutate gameplay.

---

## Implementation sequence

### 1.1 Add basic ECS components

In `bd_core`:

```text
Position { x, y }
Player
BlocksMovement
Name
```

Add map resource:

```text
SmokeMap
  width
  height
  tiles
```

Initial tile representation can be temporary:

```text
SmokeTile::Floor
SmokeTile::Wall
```

Keep constants named:

```text
SMOKE_MAP_WIDTH
SMOKE_MAP_HEIGHT
```

---

### 1.2 Add input intent

In `bd_core::signals` or `bd_core::actions`:

```text
MoveIntent
  entity
  direction

Direction
  North
  South
  East
  West
```

Input mapper in `bd_tui` or `bd_app`:

```text
W / Up    -> MoveIntent North
S / Down  -> MoveIntent South
A / Left  -> MoveIntent West
D / Right -> MoveIntent East
q         -> Quit
```

Do not move entities inside the input system.

---

### 1.3 Add movement validation and resolver

Separate systems:

```text
collect_move_intents
validate_move_intents
resolve_valid_moves
emit_log_events
```

Validation checks:

```text
destination inside map
destination not wall
destination not occupied by blocking entity
```

Movement resolver is the only system that mutates `Position`.

---

### 1.4 Add basic log

Create resource:

```text
GameLog
  entries: Vec<LogEntry>
```

Log entry:

```text
message
severity
turn/index optional
```

Use for:

```text
Moved north.
Blocked.
Cannot move there.
```

---

### 1.5 Add temporary TUI layout

In `bd_tui`:

```text
Root layout:
  top/main area
  right stat panel
  bottom log/help area
```

Widgets:

```text
SmokeMapWidget
StatsPanel
HelpLine
LogPanel
FooterVersion
```

Temporary rendering may use raw glyphs:

```text
@
#
.
```

Mark this as temporary in comments/docs until Phase 5.

---

### 1.6 Add build/version footer

Footer text:

```text
Broken Divinity dev | phase 1 | q quit
```

Version can come from Cargo env values.

---

## Tests

Unit tests in `bd_core`:

```text
valid_move_intent_changes_position
move_into_wall_is_rejected
move_out_of_bounds_is_rejected
invalid_input_does_not_mutate_world
blocked_move_emits_denial_log
```

Architecture test or convention test:

```text
movement_mutation_only_occurs_in_resolver
```

Snapshot test:

```text
smoke_map_renders_expected_ascii
```

## Phase artifact

A terminal window where:

```text
@ moves
walls block movement
HP/AP placeholders display
help line displays
log panel updates
q exits
```

---

# Phase 2 — PoolDelta Core

## Objective

Make all pool changes go through one signed delta pipeline.

---

## Implementation sequence

### 2.1 Define pool model

In `bd_core::pools`:

```text
PoolKind
  Health
  ActionPoints
  Stress
  Corruption
  Faith
  Morale

Pool
  kind
  current
  min
  max
```

Entity component:

```text
Pools
  map PoolKind -> Pool
```

Start with `Health` and `ActionPoints`.

---

### 2.2 Define pool messages

```text
PoolDeltaRequested
  source
  target
  pool
  amount
  tags
  reason

PoolDeltaApplied
  source
  target
  pool
  before
  after
  amount_applied
  tags
  reason
```

Tags can start simple:

```text
Physical
Divine
Poison
Recovery
MovementCost
```

---

### 2.3 Resolver

System:

```text
resolve_pool_deltas
```

Responsibilities:

```text
read requests
find target pool
clamp new value
write pool
emit applied event
emit threshold events
log readable result
```

Threshold events:

```text
EntityDefeated
PoolEmpty
PoolFull optional
```

---

### 2.4 Integrate movement AP

Movement should now request:

```text
ActionPoints -1
```

before movement resolves.

This creates the first cost pipeline.

Avoid direct AP mutation inside movement.

---

### 2.5 Update stat panel

Stats panel reads HP/AP from view state or temporary query.

Later Phase 6 moves this behind a formal view model.

---

## Tests

```text
health_negative_delta_reduces_health
health_positive_delta_increases_health
ap_negative_delta_spends_ap
ap_positive_delta_restores_ap
pool_delta_clamps_to_min
pool_delta_clamps_to_max
zero_delta_does_not_emit_threshold
fatal_health_emits_entity_defeated
movement_spends_ap_through_pool_delta
```

## Phase artifact

Movement consumes AP. Wait can restore AP once added or simulated.

---

# Phase 3 — Requirements, Effects, and Actions

## Objective

Convert movement, waiting, attacking, and guarding into action definitions with requirements and effects.

---

## Implementation sequence

### 3.1 Define action structures

In `bd_core::actions`:

```text
ActionId
ActionDefinition
  id
  label
  requirements
  costs
  targeting
  effects

Requirement
  HasPoolAtLeast
  TargetInRange
  TileWalkable
  TargetHostile
  EntityAlive

Effect
  PoolDelta
  MoveEntity
  ApplyStatus
  Log
```

Use Rust fixtures only.

Do not move this to RON yet.

---

### 3.2 Define action request flow

Messages:

```text
ActionIntent
  actor
  action_id
  target

ActionValidated
  actor
  action_id
  target
  resolved_definition

ActionDenied
  actor
  action_id
  reason
```

Systems:

```text
map_input_to_action_intent
validate_action_intents
resolve_action_costs
emit_action_effects
resolve_effects
```

---

### 3.3 Move action

Move becomes:

```text
Requirements:
  actor has AP >= 1
  destination is walkable

Costs:
  AP -1

Effects:
  MoveEntity direction
```

---

### 3.4 Wait action

Wait becomes:

```text
Requirements:
  actor alive

Effects:
  AP +1
  Log "You wait."
```

---

### 3.5 Attack action

Attack becomes:

```text
Requirements:
  target in range
  target hostile/alive
  actor has AP >= cost

Costs:
  AP -1 or -2

Effects:
  target Health -damage
  Log
```

Use a fixed adjacent attack first.

---

### 3.6 Guard action

Guard becomes:

```text
Costs:
  AP -1

Effects:
  ApplyStatus Guarded placeholder
```

The actual status mechanics come later.

---

### 3.7 Denial reasons

Create structured reasons:

```text
NotEnoughPool
BlockedTile
OutOfRange
NoTarget
InvalidTarget
ActorDefeated
```

UI log should display these.

---

## Tests

```text
move_denied_without_ap
move_denied_into_wall
move_costs_compile_to_pool_delta
wait_restores_ap
attack_denied_out_of_range
attack_emits_health_pool_delta
guard_emits_apply_status_effect
denial_reason_is_stable_and_displayable
```

## Phase artifact

Player can:

```text
move
wait
attack adjacent dummy/enemy
guard placeholder
see denial reasons
```

---

# Phase 4 — Schedule and Signal Discipline

## Objective

Formalize system order before statuses, modifiers, procgen, and save/load complicate the flow.

---

## Implementation sequence

### 4.1 Define schedule labels

In `bd_core::schedule`:

```text
BdSet::Input
BdSet::IntentCollection
BdSet::Validation
BdSet::CostResolution
BdSet::EffectEmission
BdSet::ModifierApplication
BdSet::Mutation
BdSet::ResultEmission
BdSet::Presentation
BdSet::ViewModelBuild
BdSet::Render
```

Register ordering explicitly.

---

### 4.2 Classify messages/events

Document and implement message classes:

```text
Intent:
  ActionIntent
  MoveIntent if still separate

Request:
  PoolDeltaRequested
  StatusApplyRequested
  SpawnRequested

Mutation/result:
  PoolDeltaApplied
  EntityMoved
  StatusApplied
  EntitySpawned

Presentation:
  LogEvent
  FlashEvent
  SelectionChanged
```

---

### 4.3 Add signal trace

Create debug resource:

```text
SignalTrace
  entries
```

Trace entry:

```text
stage
signal_type
entity optional
summary
sequence_number
```

Every important system appends trace entries in debug builds or debug mode.

---

### 4.4 Trigger-depth guard placeholder

Create resource:

```text
TriggerExecutionGuard
  current_depth
  max_depth
```

Even before full triggers exist, add the pattern.

---

### 4.5 Decide messages vs observers

Write a decision doc:

```text
Bevy messages for queued phase-based processing.
Observers only for tightly scoped immediate reactions, if used at all.
```

This prevents inconsistent event style.

---

## Tests

```text
system_order_matches_declared_schedule
mutation_happens_only_in_mutation_stage
effect_emission_does_not_mutate_position_or_pools
invalid_action_stops_before_cost_resolution
trace_records_ordered_flow
trigger_depth_guard_rejects_overflow
```

## Phase artifact

A trace can explain:

```text
input -> action intent -> validation -> cost -> effect -> mutation -> log -> render
```

---

# Phase 5 — Semantic ASCII Renderer V1

## Objective

Remove raw glyph/color knowledge from gameplay and route rendering through semantic tokens.

---

## Implementation sequence

### 5.1 Define visual tokens

In `bd_tui` or shared `bd_core::visual`:

```text
VisualToken
  Player
  Enemy
  Ally
  Floor
  Wall
  DoorClosed
  DoorOpen
  Item
  Exit
  Selection
  Fog
```

Do not overbuild the token list.

---

### 5.2 Define style tokens

```text
StyleToken
  Default
  Player
  Enemy
  Ally
  Terrain
  Wall
  Item
  Exit
  Danger
  Muted
  Selection
```

---

### 5.3 Define symbol registry

```text
SymbolDef
  visual_token
  glyph
  fallback_glyph
  layer
  style_token
  priority
```

Start in Rust fixtures.

Move to RON in Phase 8.

---

### 5.4 Define theme registry

```text
ThemeDef
  style_token -> Ratatui style properties
```

Keep actual color mapping here only.

---

### 5.5 Define render cell

```text
RenderCell
  glyph
  style_token
  layer
  source_entity optional
  tooltip optional
```

Grid:

```text
RenderCellGrid
  width
  height
  cells
```

---

### 5.6 Build map view to render grid

Flow:

```text
MapViewModel
  -> visual elements
  -> layer composition
  -> RenderCellGrid
  -> MapWidget draw
```

Gameplay entities expose semantic visual info, not glyphs.

---

### 5.7 Replace Phase 1 map widget

Retire temporary raw glyph rendering.

Temporary raw rendering should no longer be used outside test fixtures.

---

## Tests

```text
player_token_resolves_to_player_symbol
wall_token_resolves_to_wall_symbol
player_layer_renders_above_floor
enemy_layer_renders_above_item
missing_symbol_def_fails_validation
missing_style_def_fails_validation
map_snapshot_matches_expected_render
```

## Phase artifact

Rendered map looks the same or better, but implementation now uses tokens/styles.

---

# Phase 6 — View Models V1

## Objective

Prevent TUI code from reading arbitrary ECS internals.

---

## Implementation sequence

### 6.1 Define view model resources

```text
MapViewModel
StatsViewModel
ActionListViewModel
LogViewModel
ActorPanelViewModel
```

Each view model is plain data suitable for rendering.

No Bevy queries inside widgets.

---

### 6.2 Build view-model systems

Systems:

```text
build_map_view_model
build_stats_view_model
build_action_list_view_model
build_log_view_model
build_actor_panel_view_model
```

These run before render.

---

### 6.3 Restrict widgets

Widgets receive:

```text
&MapViewModel
&StatsViewModel
&ActionListViewModel
&LogViewModel
```

Widgets should not query:

```text
Position
Pools
Actions
Statuses
Inventory
```

---

### 6.4 Add available action display

Action view model should include:

```text
action label
keybinding placeholder
enabled/disabled
denial reason optional
```

---

## Tests

```text
stats_view_model_contains_hp_ap
action_list_contains_move_wait_attack_guard
disabled_action_contains_denial_reason
map_view_model_contains_only_visible_entities
widgets_can_render_from_view_models
```

## Phase artifact

TUI reads view models only. Gameplay data flow is cleaner.

---

# Phase 7 — Content IDs and Registry Core

## Objective

Prevent ad hoc string IDs before data loading expands.

---

## Implementation sequence

### 7.1 Define ContentId

In `bd_data` or shared `bd_core::ids`:

```text
ContentId
  namespace
  name
```

Rules:

```text
lowercase
dot-separated namespace
snake_case name
no spaces
no display text
```

Examples:

```text
ability.attack_basic
status.poisoned
item.rusted_spear
theme.bd_default
```

---

### 7.2 Define registry

```text
Registry<T>
  insert(id, value)
  get(id)
  contains(id)
  iter()
```

Errors:

```text
DuplicateId
MissingId
InvalidIdFormat
WrongNamespace
```

---

### 7.3 Define validation report

```text
ValidationReport
  errors
  warnings
```

Error fields:

```text
source_file optional
content_id optional
message
severity
```

---

### 7.4 Add namespace policy

Document valid namespaces:

```text
ability
status
item
blueprint
tile
theme
symbol
screen
location
faction
```

---

## Tests

```text
content_id_parses_valid_id
content_id_rejects_space
content_id_rejects_uppercase
registry_rejects_duplicate_id
registry_reports_missing_reference
validation_report_sorts_errors_deterministically
```

## Phase artifact

All future content references have a stable ID type.

---

# Phase 8 — Data Loading V1

## Objective

Move stable presentation data and only stable action data into RON.

---

## Implementation sequence

### 8.1 Add content folder conventions

Start with:

```text
content/symbols/default.ron
content/themes/default.ron
content/actions/core.ron
```

Only include `actions/core.ron` if Phase 3 schemas are stable.

---

### 8.2 Add loader entry point

In `bd_data`:

```text
ContentLoader
  load_symbols
  load_themes
  load_actions optional
```

Return:

```text
LoadedContent
  registries
  validation_report
```

---

### 8.3 Add validation passes

Separate validators:

```text
validate_ids
validate_references
validate_symbols
validate_themes
validate_actions optional
```

Do not make one giant validator function.

---

### 8.4 Add readable error output

Errors should report:

```text
file
line if available
content ID
missing reference
expected namespace
```

Line numbers may be deferred if RON tooling makes this awkward, but file and ID are required.

---

### 8.5 Schemars spike

After symbol/theme structs stabilize, test:

```text
derive schema
export schema file
compare generated schema usefulness
```

Do not block data loading on schema generation.

---

## Tests

```text
loads_valid_symbol_ron
loads_valid_theme_ron
rejects_duplicate_symbol_id
rejects_missing_style_reference
rejects_unknown_visual_token
loads_action_ron_only_when_schema_enabled
content_errors_are_deterministic
```

## Phase artifact

Symbols and themes load from disk. Action loading either works or is explicitly deferred.

---

# Phase 9 — Statuses, Triggers, and Modifiers V1

## Objective

Add runtime nuance through triggers and modifiers without duplicating combat systems.

---

## Implementation sequence

### 9.1 Define status data

```text
StatusDefinition
  id
  label
  triggers
  modifiers
  stack_policy
  default_duration

StatusInstance
  status_id
  remaining_duration
  stacks
  source optional
```

Component:

```text
Statuses
  Vec<StatusInstance>
```

---

### 9.2 Define trigger model

```text
Trigger
  OnTurnStart
  OnTurnEnd
  OnBeforePoolDelta
  OnAfterPoolDelta
  OnHit
  OnDamaged
  OnHealed
```

Triggered effect:

```text
TriggeredEffect
  requirements optional
  effects
```

---

### 9.3 Define modifier model

```text
Modifier
  target_request_type
  condition
  operation
  priority
```

First supported modifications:

```text
multiply PoolDelta amount
add flat amount
invert amount
block request
```

---

### 9.4 Add modifier ordering

Rules:

```text
sort by priority
stable tie-break by content ID
trace each modifier application
```

---

### 9.5 Implement initial statuses

```text
Poisoned:
  OnTurnStart -> Health -2 Poison

Regeneration:
  OnTurnStart -> Health +3 Recovery

Guarded:
  modifies incoming physical damage

Blessed:
  modifies positive Health deltas

Broken Choir Static:
  inverts Divine healing
```

---

### 9.6 Add loop protection

Trigger processing uses:

```text
TriggerExecutionGuard
```

Error on overflow:

```text
TriggerDepthExceeded
```

---

## Tests

```text
poison_deals_damage_on_turn_start
regeneration_heals_on_turn_start
guarded_reduces_physical_damage
blessed_increases_healing
broken_choir_static_inverts_divine_healing
status_duration_ticks_down
status_expires_at_zero
modifier_order_is_deterministic
trigger_loop_is_capped
```

## Phase artifact

Statuses work through effects and modifiers, not direct mutation.

---

# Phase 10 — Entity Factory V1

## Objective

All spawning goes through blueprints and mutators.

---

## Implementation sequence

### 10.1 Define blueprint data

```text
EntityBlueprint
  id
  components
  pools
  visual_token
  actions
  statuses
  tags
```

Keep component definitions limited at first.

---

### 10.2 Define spawn request

```text
SpawnRequest
  blueprint_id
  position optional
  faction optional
  owner optional
  location optional
  mutators
  context
```

---

### 10.3 Define mutators

```text
Mutator
  Wounded
  Elite
  FactionVariant
  TemporarySummon
```

Mutator output:

```text
component changes
pool changes
status additions
tag additions
```

---

### 10.4 Define spawn validation

Before spawning:

```text
blueprint exists
position valid if provided
mutators valid
required components present
visual token exists
```

Return:

```text
SpawnValidationReport
```

---

### 10.5 Implement factory resolver

Only factory resolver uses `Commands` to spawn blueprint-driven entities.

No enemy-specific spawn functions.

---

## Tests

```text
factory_spawns_player_blueprint
factory_spawns_enemy_blueprint
factory_spawns_item_blueprint
factory_applies_wounded_mutator
factory_applies_elite_mutator
factory_rejects_missing_blueprint
factory_rejects_invalid_position
factory_adds_visual_token
```

## Phase artifact

Player, enemy, item, trap, and exit can all be spawned through one factory path.

---

# Phase 11 — Relationships and Ownership

## Objective

Represent ownership and containment before inventory/save/transition depend on it.

---

## Implementation sequence

### 11.1 Add relationship components

```text
OwnedBy(Entity)
ContainedIn(Entity)
EquippedBy(Entity)
SummonedBy(Entity)
LocationOwned(ContentId or Entity)
FactionMember(ContentId)
```

Use narrow components first.

Do not build a generic graph relationship engine unless needed.

---

### 11.2 Add query helpers

Helper systems/functions:

```text
children_owned_by(entity)
items_contained_in(container)
equipment_for(entity)
summons_for(entity)
entities_in_location(location)
```

---

### 11.3 Add relationship validation

Validation checks:

```text
contained item has valid container
equipped item also has owner/equipped relation
summon has summoner
location-owned entity references existing location
no obvious containment cycle
```

---

## Tests

```text
summon_has_summoner
item_can_be_contained
item_can_be_equipped
equipped_item_can_be_queried
contained_item_can_be_queried
containment_cycle_is_rejected
location_owned_entity_can_be_queried
```

## Phase artifact

Ownership is explicit enough for inventory and save/load.

---

# Phase 12 — Inventory, Equipment, and Containers

## Objective

Create generic containers and item interactions.

---

## Implementation sequence

### 12.1 Define item policy

Choose one:

```text
items are entities
```

Use item entities for flexibility.

Item entity components:

```text
Item
Name
VisualToken
ContainedIn optional
EquippedBy optional
Usable optional
```

---

### 12.2 Define container

```text
Container
  capacity optional
  allowed_tags optional
```

Player inventory is an entity or component with `Container`.

Chests, corpses, shops later can also be containers.

---

### 12.3 Define equipment slots

```text
EquipmentSlot
  slot_kind
  accepted_tags
```

Slot kinds:

```text
Weapon
Armor
Relic
Accessory optional
```

---

### 12.4 Define inventory intents

```text
PickupIntent
DropIntent
EquipIntent
UnequipIntent
UseItemIntent
TransferItemIntent
```

All become action/effect requests.

---

### 12.5 Define item effects

Usable item:

```text
UseItemEffect
  effects
  consume_on_use
```

Example:

```text
healing draught -> Health +5
```

No item directly mutates pools.

---

### 12.6 Build container view model

```text
ContainerViewModel
  container name
  item rows
  equipped markers
  usable markers
```

---

## Tests

```text
pickup_moves_item_into_inventory_container
drop_moves_item_to_map_position
equip_moves_item_to_slot
unequip_returns_item_to_inventory
invalid_slot_is_rejected
use_item_emits_effects
consumable_item_is_removed_after_use
container_view_model_lists_items
```

## Phase artifact

Player can pick up, drop, equip, unequip, and use items through the same intent/effect pipeline.

---

# Phase 13 — Pathfinding and Visibility Spike

## Objective

Pick pathing/FOV implementation and hide it behind adapters.

---

## Implementation sequence

### 13.1 Define adapter traits

```text
Pathfinder
  find_path(map, start, goal) -> PathResult

MovementRangeProvider
  reachable_tiles(map, origin, budget) -> TileSet

VisibilityProvider
  visible_tiles(map, origin, radius) -> TileSet
```

---

### 13.2 Test `pathfinding` crate

Implement adapter for:

```text
A*
Dijkstra/reachable tiles if useful
```

Test weighted movement.

---

### 13.3 Test `bracket-pathfinding`

Only test features not covered cleanly by `pathfinding`:

```text
FOV
Dijkstra maps
roguelike grid helpers
```

---

### 13.4 Record decision

Decision doc:

```text
docs/decisions/pathfinding-fov.md
```

Include:

```text
chosen crate
rejected crate
why
adapter boundary
fallback
```

---

### 13.5 Integrate movement range overlay

Add view-model data:

```text
MovementRangeOverlay
```

TUI renders it through semantic tokens.

---

## Tests

```text
path_avoids_walls
path_returns_none_when_unreachable
movement_range_respects_ap_budget
occupied_blocking_tile_blocks_path
visibility_hides_unseen_enemy
remembered_tile_uses_muted_visual_state
```

## Phase artifact

Movement range and visibility are available through adapters, not hardcoded algorithms.

---

# Phase 14 — Procedural Location V1

## Objective

Generate location plans, validate them, then spawn from plan.

---

## Implementation sequence

### 14.1 Define location template

```text
LocationTemplate
  id
  size_range
  room_count_range
  tile_palette
  spawn_table
  exit_rules
```

---

### 14.2 Define generated plan

```text
LocationPlan
  seed
  width
  height
  tiles
  rooms
  spawn_zones
  exits
  entrance
```

No entities spawned yet.

---

### 14.3 Generation pipeline

Functions/stages:

```text
create_base_layout
place_rooms
connect_rooms
paint_tiles
place_entrance
place_exits
place_spawn_zones
validate_plan
convert_to_spawn_requests
```

---

### 14.4 Validation

Plan validator checks:

```text
all rooms reachable
entrance exists
exit exists
exit reachable from entrance
spawn zones on valid tiles
no spawn zone in wall
minimum walkable area
```

---

### 14.5 Spawn integration

Only after validation:

```text
LocationPlan -> SpawnRequests -> FactoryResolver
```

---

## Tests

```text
same_seed_generates_same_plan
different_seed_generates_different_plan
all_rooms_reachable
entrance_exists
exit_reachable
spawn_zones_valid
plan_does_not_spawn_entities_before_validation
```

## Phase artifact

A generated ruin/dungeon can be entered and traversed.

---

# Phase 15 — Data-Driven TUI Screens

## Objective

Move screen layout into data after view models are stable.

---

## Phase 15A — Combat screen

### Implementation sequence

Define:

```text
ScreenDefinition
PanelDefinition
WidgetBinding
WidgetRegistry
ScreenState
```

Combat screen data:

```text
map panel
stats panel
actions panel
log panel
help/footer panel
```

Widget registry maps IDs to widget constructors/renderers.

Validation checks:

```text
widget ID exists
view model binding exists
layout region valid
duplicate panel ID rejected
```

### Tests

```text
combat_screen_loads
combat_screen_resolves_widgets
missing_widget_id_fails
missing_view_model_binding_fails
combat_screen_renders_snapshot
```

---

## Phase 15B — Inventory screen

### Implementation sequence

Add:

```text
inventory screen definition
screen switching action/intent
inventory container binding
equipment panel binding
```

### Tests

```text
inventory_screen_loads
inventory_screen_resolves_container_view_model
screen_switch_preserves_gameplay_state
missing_inventory_binding_fails
```

## Phase artifact

At least two screens prove screen definitions are reusable.

---

# Phase 16 — Config, Preferences, and App Directories

## Objective

Externalize app config, paths, and user preferences without hand-rolled OS logic.

---

## Implementation sequence

### 16.1 Pick directory crate

Spike:

```text
directories / ProjectDirs
```

Define app identity:

```text
qualifier
organization
application
```

Map paths:

```text
config directory
data directory
save directory
log directory
cache directory optional
```

---

### 16.2 Define config structs

```text
AppConfig
  theme_id
  symbol_mode
  keybindings
  save_dir_override
  log_level
  debug_flags
```

Keybindings:

```text
action_id -> key/chord
```

---

### 16.3 Config loading

Flow:

```text
load default config
load user config if exists
merge
validate
apply
```

Create default config or use built-in defaults when user config is missing.

---

### 16.4 Help line from bindings

Replace hardcoded Phase 1 help line.

Help line should be generated from:

```text
KeyBindingConfig
ActionRegistry
```

---

### 16.5 Settings persistence spike

Test:

```text
bevy-persistent
bevy_persist
```

Only for settings/resources, not full world saves.

---

## Tests

```text
default_config_loads
missing_config_uses_defaults
bad_config_reports_readable_error
config_directory_resolves
save_directory_resolves
keybinding_maps_to_action
help_line_derives_from_bindings
settings_persist_across_app_restart_simulation
```

## Phase artifact

Config and paths are platform-correct and no longer hardcoded.

---

# Phase 17 — Save / Load / Replay Spikes

## Objective

Choose persistence approach after entity identity, ownership, inventory, and location state exist.

---

## Implementation sequence

### 17.1 Define save boundary

Types:

```text
PersistentEntity
TransientEntity
SaveExcluded
SaveId
ContentVersion
SaveVersion
```

Rules:

```text
content IDs are saved
runtime entity IDs are not trusted across loads
save IDs map restored entities
transient summons excluded unless marked persistent
```

---

### 17.2 Define snapshot shape

```text
RunSnapshot
  save_version
  content_version
  current_location
  player_state
  entities
  inventories
  equipment
  location_seed
  turn_state

LocationSnapshot
  location_id
  seed
  tiles or generator params
  persistent entities
```

---

### 17.3 Spike save candidates

First candidate:

```text
bevy_save
```

Second candidate:

```text
moonshine-save
```

Test only the project’s required shape.

Do not compare five libraries at once.

---

### 17.4 Custom replay log

Define:

```text
IntentReplayLog
  seed
  initial_snapshot_ref optional
  ordered intents
```

Replay deterministic sequence:

```text
start fixed seed
apply fixed intents
assert final state
```

---

### 17.5 Load validation

On load:

```text
check save version
check content version
check required content IDs exist
restore entities
reconnect relationships
validate snapshot
```

Do not apply gameplay effects during load.

---

## Tests

```text
player_position_persists
pools_persist
inventory_persists
equipment_persists
location_seed_persists
transient_summon_excluded
save_version_recorded
content_version_mismatch_errors_or_migrates
relationships_restore
fixed_intent_replay_is_deterministic
```

## Phase artifact

Accepted persistence strategy with a working small-world save/load roundtrip.

---

# Phase 18 — Broken Divinity Tactical MVP

## Objective

Build a compact Broken Divinity tactical loop using the kernel.

---

## Implementation sequence

### 18.1 Content pack

Create data files:

```text
content/bd_mvp/actions.ron
content/bd_mvp/statuses.ron
content/bd_mvp/items.ron
content/bd_mvp/blueprints.ron
content/bd_mvp/location_templates.ron
content/bd_mvp/screens.ron
```

Content targets:

```text
player archetype
two enemies
one summon/ally
five abilities
five statuses
five items
three tile types
one location template
```

---

### 18.2 MVP loop integration

Flow:

```text
new run
spawn player
generate ruin
enter location
fight enemies
collect loot
reach exit
return to placeholder outpost
save/load
```

---

### 18.3 Debug trace coverage

Every major action should trace:

```text
intent
validation
cost
effect
modifier
mutation
result
log
```

---

## Tests

```text
mvp_run_can_start
mvp_location_generates
player_can_kill_enemy
enemy_can_kill_player
player_can_pick_up_loot
player_can_reach_exit
mvp_save_load_roundtrip
```

## Phase artifact

Playable 10-minute tactical slice.

---

# Phase 19 — Outpost, Travel, and Transitions V1

## Objective

Connect tactical locations to a basic outpost/travel layer.

---

## Implementation sequence

### 19.1 Define mode/state

```text
GameMode
  Outpost
  Travel
  TacticalLocation
```

Transitions:

```text
EnterLocation
ExitLocation
ReturnToOutpost
StartTravel
CompleteTravel
```

---

### 19.2 Outpost state

```text
OutpostState
  resources
  party
  storage container
  production timers
```

Use PoolDelta-like resource changes where applicable.

---

### 19.3 Travel nodes

```text
TravelNode
TravelRoute
TravelTime
```

Keep simple. No full world map yet.

---

### 19.4 State isolation

Ensure tactical-only entities do not leak into outpost.

Persistent entities:

```text
player
party
inventory
outpost storage
location memory
```

Transient entities:

```text
temporary combat enemies
temporary summons
temporary effects
```

---

## Tests

```text
enter_location_preserves_player
exit_location_returns_to_outpost
travel_advances_time
outpost_resource_changes_use_pool_like_path
transient_combat_entity_does_not_leak
storage_container_persists
```

## Phase artifact

The tactical game connects to a simple outpost/travel loop.

---

# Phase 20 — UX, Debugging, and Tooling Hardening

## Objective

Make the project usable, diagnosable, and safe to develop content for.

---

## Implementation sequence

### 20.1 Debug overlay

Add toggle:

```text
F1 debug overlay
```

Overlay data:

```text
entity under cursor
position
visual token
pools
statuses
content ID
```

Read-only by default.

---

### 20.2 Entity inspector

Add debug screen/panel:

```text
selected entity
components summary
relationships
inventory/equipment
trace history optional
```

Any mutation requires DebugIntent.

---

### 20.3 Event trace viewer

Allow browsing recent trace entries:

```text
filter by entity
filter by signal type
filter by stage
```

---

### 20.4 Content validation CLI

Command:

```text
bd_app validate-content
```

or separate tool later.

Validates:

```text
IDs
references
symbols
themes
actions
statuses
blueprints
screens
location templates
```

---

### 20.5 Preview tools

Commands or debug screens:

```text
preview-procgen --seed
preview-theme
preview-symbols
```

---

### 20.6 Panic/error reporting

Finalize:

```text
terminal cleanup
panic report
log file path
last trace dump optional
```

---

## Tests/checks

```text
debug_overlay_reads_only
debug_mutation_requires_debug_intent
validator_catches_missing_reference
validator_catches_bad_blueprint
procgen_preview_uses_seed
panic_path_restores_terminal
```

## Phase artifact

Content mistakes and runtime bugs are easy to diagnose.

---

# Phase 21 — Performance and Stability

## Objective

Measure real bottlenecks and optimize only what matters.

---

## Implementation sequence

### 21.1 Add metrics

Track:

```text
frame time
render time
view-model build time
pathfinding time
procgen time
save/load time
event queue size
entity count
trigger depth
```

---

### 21.2 Add stability simulations

Run deterministic tests:

```text
100-turn combat simulation
seed batch procgen
repeated save/load
large event-chain scenario
inventory transfer stress
```

---

### 21.3 Optimize only measured issues

Possible work:

```text
cache view models
cache pathfinding
cache visibility
reduce render allocations
dirty-cell rendering
registry indexing
log batching
```

Each optimization requires before/after measurement.

---

## Tests

```text
hundred_turn_simulation_does_not_leak_entities
event_queue_does_not_grow_unbounded
seed_batch_does_not_panic
save_load_stress_roundtrip_passes
render_snapshot_remains_stable_after_optimization
```

## Phase artifact

The game is stable under repeated play and deterministic stress.

---

# Phase 22 — Packaging and Release Candidate

## Objective

Run the game outside the dev environment.

---

## Implementation sequence

### 22.1 Release layout

Package:

```text
bd_app binary
content/
config/default.toml
README.md
LICENSE
CHANGELOG optional
```

Runtime creates:

```text
save directory
log directory
user config directory
```

---

### 22.2 Release smoke script

Script should:

```text
run binary
load default config
load content
start fixed-seed run
execute short intent script
quit cleanly
verify log file
```

---

### 22.3 Documentation

README includes:

```text
how to run
controls
config path
save path
logs path
known issues
troubleshooting terminal problems
```

---

## Checks

```text
fresh_checkout_builds
release_binary_runs
content_pack_loads
config_loads
save_dir_created
log_file_written
terminal_exits_cleanly
```

## Phase artifact

A release candidate that can be run from a clean folder.

---

# Phase 23 — Standalone Roguelike Prototype Approval

## Objective

Validate kernel reuse without creating a second product.

---

## Implementation sequence

### 23.1 Create separate content pack

```text
content/roguelike_proto/
  actions.ron
  statuses.ron
  items.ron
  blueprints.ron
  locations.ron
  screens.ron
```

No new core systems.

---

### 23.2 Prototype content

Minimum content:

```text
one player
two enemies
one special ability
two statuses
three to five items
one weapon type
one armor/relic slot
four tile types
one dungeon template
two floors
one final encounter
```

---

### 23.3 Prototype loop

```text
start run
generate floor 1
explore/fight/loot
descend
generate floor 2
final encounter
win or die
summary screen
```

---

### 23.4 Fixed-seed validation

Create deterministic test:

```text
seed
intent script
expected final summary
```

This is the real approval test.

---

### 23.5 No-polish enforcement

Reject work that adds:

```text
prototype-only UI
prototype-only combat
prototype-only procgen
special-case save/load
extra content beyond validation target
```

---

## Tests

```text
prototype_starts
floor_one_reachable
floor_two_reachable
final_encounter_reachable
prototype_player_can_win_or_die
prototype_save_load_roundtrip
prototype_fixed_seed_replay_matches_expected_summary
```

## Phase artifact

The reusable kernel powers a second small game loop without hacks.

---

# Phase 24 — Production-Ready Gate

## Objective

Approve the kernel and Broken Divinity MVP for larger production.

---

## Gate checklist

```text
all tests pass
content validator passes
terminal cleanup reliable
save/load roundtrip passes
BD tactical MVP works
outpost/travel transition works
roguelike prototype works
no prototype-only hacks
dependencies pinned
release build runs from clean folder
known issues documented
```

---

## Final deliverables

```text
working release build
validated content packs
saved-run compatibility note
DecisionLog complete
architecture docs current
known issues list
developer runbook
```

---

# Final Execution Rule

Do not start a phase by writing the “main” implementation first.

For each phase:

```text
1. write the smallest failing test or smoke check
2. implement the smallest slice
3. pass the test
4. wire into the app
5. add snapshot/integration coverage
6. update docs/decision log
7. verify exit criteria
```

Do not compress phases.

Do not proceed past failed exit criteria.

Do not build custom infrastructure before the relevant external crate has failed a documented spike.
