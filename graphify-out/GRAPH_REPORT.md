# Graph Report - .  (2026-04-10)

## Corpus Check
- 86 files · ~76,722 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 896 nodes · 1236 edges · 93 communities detected
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 39 edges (avg confidence: 0.79)
- Token cost: 0 input · 0 output

## God Nodes (most connected - your core abstractions)
1. `generate_floor()` - 14 edges
2. `autosave System (On Colony Enter)` - 13 edges
3. `handle_save_and_quit System` - 13 edges
4. `RaidExposure Component (Sanity Meter)` - 12 edges
5. `CombatStats Component` - 11 edges
6. `calc_damage Function` - 11 edges
7. `Inventory Component (20 Slots)` - 10 edges
8. `StatusEffects Component` - 10 edges
9. `PlayerPerks` - 9 edges
10. `grid_movement System (WASD/Bump Attack)` - 8 edges

## Surprising Connections (you probably didn't know these)
- `Save/Load Interface (5 Slots)` --semantically_similar_to--> `autosave System (On Colony Enter)`  [INFERRED] [semantically similar]
  docs/ui/menus.md → src/core/save.rs
- `RaidExposure Component (Sanity Meter)` --semantically_similar_to--> `Travel System (Time/Food/Water/Danger)`  [INFERRED] [semantically similar]
  src/core/sanity.rs → docs/gameplay/overworld.md
- `Sanity Distortion System (UI Lies)` --semantically_similar_to--> `RaidExposure Component (Sanity Meter)`  [INFERRED] [semantically similar]
  docs/ui/README.md → src/core/sanity.rs
- `LogColor Semantic Colors` --semantically_similar_to--> `UI Color Palette`  [INFERRED] [semantically similar]
  src/core/gamelog.rs → docs/ui/README.md
- `Sanity Distortion System (UI Lies)` --semantically_similar_to--> `SanityThreshold Enum (4 Bands)`  [INFERRED] [semantically similar]
  docs/ui/README.md → src/core/sanity.rs

## Hyperedges (group relationships)
- **Core Game Loop (Shelter → Overworld → Dungeon)** — gdd_game_loop, colony_shelter_layout, overworld_node_graph, procgen_bsp_generation, combat_d100_system, colony_resources [EXTRACTED 0.95]
- **Deterministic Procgen Pipeline (Seed → ChaCha8 → World)** — procgen_seed_architecture, architecture_chacha8rng, procgen_bsp_generation, procgen_overworld_gen, procgen_faction_gen, procgen_determinism [EXTRACTED 0.90]
- **Combat System Reused Across Dungeon and Shelter Raids** — combat_d100_system, combat_action_budget, colony_raids, combat_shelter_raids, colony_survivor_presets [EXTRACTED 0.90]
- **Player State Handoff Across AppStates** — main_enter_overworld_from_colony, save_player_snapshot, save_handle_save_and_quit, save_autosave, player_bundle [EXTRACTED 0.90]
- **Movement→FOV→Camera System Chain** — movement_grid_movement, movement_sync_transform, fov_update_viewshed, camera_setup, mod_system_chain [EXTRACTED 1.00]
- **Sanity↔Perks↔Movement Cross-Cutting Interaction** — sanity_raid_exposure, sanity_check_control_loss, movement_grid_movement, perks_player_perks, sanity_apply_player_event [EXTRACTED 0.90]
- **d100 Combat Resolution Pipeline** — combat_roll_check, combat_calc_damage, stats_combatstats, combat_damagetype, dungeon_melee_resolve_player_melee, dungeon_melee_resolve_enemy_melee, dungeon_ai_enemy_ai_turn, dungeon_gabriel_gabriel_turn [EXTRACTED 0.95]
- **Dungeon Room Content Spawning Pattern** — dungeon_bsp_rect, dungeon_anomalies_spawn_anomalies, dungeon_hazards_spawn_hazards, dungeon_loot_spawn_loot_in_rooms, dungeon_lore_spawn_lore_drops, dungeon_enemies_spawn_enemy [INFERRED 0.90]
- **Colony Shelter Initialization Chain** — colony_spawn_setup_shelter, colony_mapgen_generate_shelter, colony_stations_spawn_station, colony_survivors_spawn_initial, colony_spawn_cleanup_shelter [EXTRACTED 0.95]
- **Overworld Travel Pipeline** — overworld_map_draw_overworld_map, overworld_map_SelectedDestination, overworld_mod_start_travel, overworld_travel_TravelState, overworld_travel_process_travel_day, overworld_weather_roll_weather, overworld_mod_handle_arrival [EXTRACTED 0.90]
- **egui Draw/Process Split Pattern** — ui_colony_panel_ColonyUiAction, ui_gabriel_dialogue_panel_GabrielDialogueUiAction, ui_menu_MenuUiAction, ui_overworld_panel_OverworldUiAction, ui_gameover_DeathSummary [EXTRACTED 0.95]
- **Dungeon ↔ Overworld State Bridge** — overworld_mod_handle_arrival, dungeon_spawn_DungeonState, dungeon_spawn_seed_for_dungeon_site, dungeon_spawn_handle_stairs, overworld_graphgen_NodeType, dungeon_theme_DungeonTheme [EXTRACTED 0.85]

## Communities

### Community 0 - "combat.rs / calc_damage Function / CheckResult Struct"
Cohesion: 0.05
Nodes (38): calc_damage Function, CheckResult Struct, DamageType Enum, roll_check d100 Function, test_crit_doubles_damage(), test_damage_min_1(), test_fumble_on_100(), test_roll_check_success() (+30 more)

### Community 1 - "Save/Load Interface (5 Slots) / save.rs / autosave System (On Colony Enter)"
Cohesion: 0.06
Nodes (42): Save/Load Interface (5 Slots), autosave System (On Colony Enter), delete_save(), handle_save_and_quit System, insert_legacy_skill(), Save Legacy Field Migration, legacy_resources_present(), load_game() (+34 more)

### Community 2 - "abilities.rs / calc_cover (Cover Level Calculation) / calc_range_penalty()"
Cohesion: 0.04
Nodes (46): calc_cover (Cover Level Calculation), CoverLevel Enum (None/Half/Full), CoverLevel, floor_map(), is_wall(), calc_range_penalty Function, SprintCooldown, test_cover_full() (+38 more)

### Community 3 - "Hand-Rolled FOV (Symmetric Shadowcasting ~150 LOC) / Colony Loop Rhythm (Return→Deposit→Repair→Prepare) / Survivor Needs (Hunger/Thirst/Rest)"
Cohesion: 0.04
Nodes (64): Hand-Rolled FOV (Symmetric Shadowcasting ~150 LOC), Colony Loop Rhythm (Return→Deposit→Repair→Prepare), Survivor Needs (Hunger/Thirst/Rest), Walkable Shelter Compound (BSP-variant), Survivor Workforce (3-5 NPCs), Speed & Action Budget System, Cover System (Half/Full), Why d100 (Granularity for Modifier Stacking) (+56 more)

### Community 4 - "camera.rs / camera_follow() / setup_camera()"
Cohesion: 0.04
Nodes (31): Inventory Panel (Grid UI), ArmorDurability, Equipment Component (Weapon/Armor/Accessory), Inventory Component (20 Slots), InventoryOpen, InventoryUiAction, InventoryUiChoice, RangedWeaponState (+23 more)

### Community 5 - "LoreCategory Enum / factions.rs / Faction"
Cohesion: 0.06
Nodes (27): LoreCategory Enum, Faction, FactionArchetype Enum, FactionDisposition, Factions Resource, generate_factions Function, test_determinism(), test_faction_count() (+19 more)

### Community 6 - "Dungeon Module Plugin / ShootTarget Resource / Handle Reload Input System"
Cohesion: 0.07
Nodes (42): Dungeon Module Plugin, ShootTarget Resource, Handle Reload Input System, Handle Shoot Input System, Resolve Ranged Attack System, DungeonState Resource, Configure Story Entities, Handle Stairs System (+34 more)

### Community 7 - "generate_shelter Function / ShelterRoomKind Enum / Colony Plugin"
Cohesion: 0.07
Nodes (32): generate_shelter Function, ShelterRoomKind Enum, Colony Plugin, auto_resolve_raid Function, check_raid_trigger System, resolve_active_raid System, cleanup_shelter System, setup_shelter System (+24 more)

### Community 8 - "Character Sheet Panel / perks.rs / apply_second_wind Function"
Cohesion: 0.1
Nodes (16): Character Sheet Panel, apply_second_wind Function, make_stats(), PendingPerkChoices, PerkId Enum (9 Perks / 3 Trees), PerkId, PlayerPerks Component, PlayerPerks (+8 more)

### Community 9 - "resources.rs / ResourceKind / ShelterResources"
Cohesion: 0.08
Nodes (15): ResourceKind, ShelterResources, test_add(), test_new_game_resources(), test_try_consume_fail(), test_try_consume_success(), WorldSeed, EncounterType (+7 more)

### Community 10 - "gabriel.rs / companion_spawn_on_floor() / gabriel_dialogue_panel.rs"
Cohesion: 0.08
Nodes (19): companion_spawn_on_floor(), accept_label(), dialogue_text(), draw_gabriel_dialogue_panel(), GabrielDialogueUiAction, GabrielDialogueUiChoice, Gabriel, gabriel_turn() (+11 more)

### Community 11 - "anomalies.rs / Anomaly / AnomalyKind"
Cohesion: 0.09
Nodes (17): Anomaly, AnomalyKind, check_anomaly_proximity(), EnemyDef, RangedEnemy, RangedEnemyData, configure_story_entities(), DungeonState (+9 more)

### Community 12 - "ai.rs / enemy_ai_turn() / has_los()"
Cohesion: 0.08
Nodes (12): enemy_ai_turn(), has_los(), GameLog, LogColor, LogEntry, draw_gamelog_panel(), gamelog_color(), check_hazard_tiles() (+4 more)

### Community 13 - "colony_panel.rs / ColonyUiAction / ColonyUiChoice"
Cohesion: 0.1
Nodes (17): ColonyUiAction, ColonyUiChoice, draw_resource_bar(), draw_survivor_panel(), need_bar(), resource_label(), task_label(), CriticalNeed (+9 more)

### Community 14 - "raids.rs / ActiveRaid / auto_resolve_raid()"
Cohesion: 0.09
Nodes (16): ActiveRaid, auto_resolve_raid(), CombatPreset, RaidChance, RaidPhase, RaidReport, resolve_active_raid(), test_auto_resolve_strong_defense() (+8 more)

### Community 15 - "sanity.rs / apply_player_sanity_event() / apply_sanity_event()"
Cohesion: 0.14
Nodes (15): apply_player_sanity_event(), apply_sanity_event(), check_control_loss System, check_hallucinations System, forced_move_direction(), Hallucination Component, RaidExposure, SanityEvent (+7 more)

### Community 16 - "bsp.rs / BspNode / .collect_leaves()"
Cohesion: 0.21
Nodes (13): BspNode, carve_corridor(), carve_room(), connect_bsp(), DungeonFloor, generate_floor(), place_doors(), Rect (+5 more)

### Community 17 - "bevy_ecs_tilemap 0.18.1 (GPU Tilemap) / bevy_egui 0.39.1 (UI Framework) / egui Draw/Process Split Pattern"
Cohesion: 0.11
Nodes (19): bevy_ecs_tilemap 0.18.1 (GPU Tilemap), bevy_egui 0.39.1 (UI Framework), egui Draw/Process Split Pattern, Three-Layer Rendering Pipeline, Raid System (Event-Driven Shelter Defense), 5 Core Resources (Food/Water/Scrap/Medicine/Ammo), Schrödinger's Raid (Auto-Resolve When Away), 10 Station Types (T1 MVP) (+11 more)

### Community 18 - "mapgen.rs / carve_corridor() / carve_room()"
Cohesion: 0.27
Nodes (13): carve_corridor(), carve_room(), generate_shelter(), in_bounds(), place_doors(), ShelterData, ShelterRoom, ShelterRoomKind (+5 more)

### Community 19 - "lore.rs / all_fragments() / LoreCategory"
Cohesion: 0.21
Nodes (10): all_fragments(), LoreCategory, LoreDrop, LoreFragment, LoreJournal, pickup_lore(), spawn_lore_drops(), test_all_fragments_unique() (+2 more)

### Community 20 - "GameLog Resource (Ring Buffer) / Combat Feedback (Floaters/Flashes) / Hunger/Thirst Badge System"
Cohesion: 0.2
Nodes (10): GameLog Resource (Ring Buffer), Combat Feedback (Floaters/Flashes), Hunger/Thirst Badge System, Vitals Bar (HP/Sanity/AP/Ammo), UI Design Philosophy, Feedback Principles (Visual Before Verbal), Map Is King Rationale, Panel System (UiPanel, UiNavState) (+2 more)

### Community 21 - "gameover.rs / check_player_death() / DeathSummary"
Cohesion: 0.29
Nodes (3): DeathSummary, GameOverUiAction, GameOverUiChoice

### Community 22 - "lib.rs / test_same_seed_same_dungeon() / test_same_seed_same_factions()"
Cohesion: 0.33
Nodes (0): 

### Community 23 - "Construction Queue Tab / Shelter Management Window (7 Tabs) / Post-Raid Report Modal"
Cohesion: 0.33
Nodes (6): Construction Queue Tab, Shelter Management Window (7 Tabs), Post-Raid Report Modal, Raid Prep Tab (Contextual), Stations Tab (T1 Stations), Survivors Tab (Cards/Needs)

### Community 24 - "menu.rs / draw_main_menu() / MenuUiAction"
Cohesion: 0.4
Nodes (2): MenuUiAction, MenuUiChoice

### Community 25 - "5-Tier Module Dependency Graph / Data-Driven Content (RON + OnceLock) / Message Pipeline (Decoupled Communication)"
Cohesion: 0.4
Nodes (5): 5-Tier Module Dependency Graph, Data-Driven Content (RON + OnceLock), Message Pipeline (Decoupled Communication), Save Compatibility (serde defaults), Scalability Design (5 Mechanisms)

### Community 26 - "journal_panel.rs / draw_journal_panel() / JournalOpen"
Cohesion: 0.5
Nodes (1): JournalOpen

### Community 27 - "theme.rs / DungeonTheme / .atlas_index()"
Cohesion: 0.5
Nodes (1): DungeonTheme

### Community 28 - "ChaCha8Rng (Deterministic Cross-Platform RNG) / Weather System (8 Types, Deterministic) / Determinism Contracts (Same Seed = Same Output)"
Cohesion: 0.5
Nodes (4): ChaCha8Rng (Deterministic Cross-Platform RNG), Weather System (8 Types, Deterministic), Determinism Contracts (Same Seed = Same Output), Deterministic Seed Architecture (u64 World Seed)

### Community 29 - "MVP Dev Plan (9 Vertical Slices) / MVP Phase — 'The Bones' / Phase 2 — Colony Foundation"
Cohesion: 0.5
Nodes (4): MVP Dev Plan (9 Vertical Slices), MVP Phase — 'The Bones', Phase 2 — Colony Foundation, Phase 3 — Full Colony (DF/RimWorld Vision)

### Community 30 - "4 Core Abilities (Attack/Shoot/FirstAid/Sprint) / Universal Ammo System (Clip+Reload) / [HIGH] Stale ShootTarget on Dungeon Re-entry"
Cohesion: 0.67
Nodes (3): 4 Core Abilities (Attack/Shoot/FirstAid/Sprint), Universal Ammo System (Clip+Reload), [HIGH] Stale ShootTarget on Dungeon Re-entry

### Community 31 - "Overworld Encounters / Travel Mode (Progress/Encounters) / Weather System (8 Types)"
Cohesion: 0.67
Nodes (3): Overworld Encounters, Travel Mode (Progress/Encounters), Weather System (8 Types)

### Community 32 - "ResourceKind Enum / ShelterResources (5 Resource Types) / Shelter Resource Bar"
Cohesion: 0.67
Nodes (3): ResourceKind Enum, ShelterResources (5 Resource Types), Shelter Resource Bar

### Community 33 - "GabrielDialogueUiAction Resource / Draw Gabriel Dialogue Panel / Process Gabriel Dialogue Action"
Cohesion: 1.0
Nodes (3): GabrielDialogueUiAction Resource, Draw Gabriel Dialogue Panel, Process Gabriel Dialogue Action

### Community 34 - "MenuUiAction Resource / Draw Main Menu System / Process Menu Action System"
Cohesion: 1.0
Nodes (3): MenuUiAction Resource, Draw Main Menu System, Process Menu Action System

### Community 35 - "state.rs / AppState"
Cohesion: 1.0
Nodes (1): AppState

### Community 36 - "tilemap.rs / spawn_tilemap_layer Function"
Cohesion: 1.0
Nodes (1): spawn_tilemap_layer Function

### Community 37 - "Weighted Loot Tables (5 Quality Tiers) / Equipment Tier System (T0-T3)"
Cohesion: 1.0
Nodes (2): Weighted Loot Tables (5 Quality Tiers), Equipment Tier System (T0-T3)

### Community 38 - "Bevy 0.18.1 (ECS Engine) / Bevy 2d Feature Flag (Eliminate 3D Pipeline)"
Cohesion: 1.0
Nodes (2): Bevy 0.18.1 (ECS Engine), Bevy 2d Feature Flag (Eliminate 3D Pipeline)

### Community 39 - "Node Info Panel / World Node Map (Graph Not Grid)"
Cohesion: 1.0
Nodes (2): Node Info Panel, World Node Map (Graph Not Grid)

### Community 40 - "LogColor Semantic Colors / UI Color Palette"
Cohesion: 1.0
Nodes (2): LogColor Semantic Colors, UI Color Palette

### Community 41 - "ItemDef Static Definition / WeaponProps (Damage/Range/Accuracy)"
Cohesion: 1.0
Nodes (2): ItemDef Static Definition, WeaponProps (Damage/Range/Accuracy)

### Community 42 - "Determinism Test Suite / WorldSeed Resource"
Cohesion: 1.0
Nodes (2): Determinism Test Suite, WorldSeed Resource

### Community 43 - "EnemyDef Struct / spawn_table Function"
Cohesion: 1.0
Nodes (2): EnemyDef Struct, spawn_table Function

### Community 44 - "DungeonFloor Struct / GabrielState Resource"
Cohesion: 1.0
Nodes (2): DungeonFloor Struct, GabrielState Resource

### Community 45 - "pickup_items System / pickup_lore System"
Cohesion: 1.0
Nodes (2): pickup_items System, pickup_lore System

### Community 46 - "JournalOpen Resource / Draw Journal Panel"
Cohesion: 1.0
Nodes (2): JournalOpen Resource, Draw Journal Panel

### Community 47 - "Draw Perk Choice Panel / Process Perk Choice Action"
Cohesion: 1.0
Nodes (2): Draw Perk Choice Panel, Process Perk Choice Action

### Community 48 - "Player Background System"
Cohesion: 1.0
Nodes (1): Player Background System

### Community 49 - "Permadeath & Game Modes"
Cohesion: 1.0
Nodes (1): Permadeath & Game Modes

### Community 50 - "Endgame Dual Paths (Sandbox vs Truth)"
Cohesion: 1.0
Nodes (1): Endgame Dual Paths (Sandbox vs Truth)

### Community 51 - "Status Effects (Wounded, Stunned)"
Cohesion: 1.0
Nodes (1): Status Effects (Wounded, Stunned)

### Community 52 - "Noise & Detection (MVP Simplified)"
Cohesion: 1.0
Nodes (1): Noise & Detection (MVP Simplified)

### Community 53 - "Survivors (Player Faction)"
Cohesion: 1.0
Nodes (1): Survivors (Player Faction)

### Community 54 - "Angelic vs Infernal Sensory Comparison"
Cohesion: 1.0
Nodes (1): Angelic vs Infernal Sensory Comparison

### Community 55 - "Dungeon Theme Mixing (Zone Transitions)"
Cohesion: 1.0
Nodes (1): Dungeon Theme Mixing (Zone Transitions)

### Community 56 - "Phase Dependency Chain"
Cohesion: 1.0
Nodes (1): Phase Dependency Chain

### Community 57 - "Dev Slice Dependency Graph (Sequential)"
Cohesion: 1.0
Nodes (1): Dev Slice Dependency Graph (Sequential)

### Community 58 - "98/98 Tests Pass (Build Verified)"
Cohesion: 1.0
Nodes (1): 98/98 Tests Pass (Build Verified)

### Community 59 - "Typography System"
Cohesion: 1.0
Nodes (1): Typography System

### Community 60 - "Dungeon & Shelter Theming"
Cohesion: 1.0
Nodes (1): Dungeon & Shelter Theming

### Community 61 - "Escape Cascade Priority Chain"
Cohesion: 1.0
Nodes (1): Escape Cascade Priority Chain

### Community 62 - "InGame Game Log Panel"
Cohesion: 1.0
Nodes (1): InGame Game Log Panel

### Community 63 - "Ability Picker Panel"
Cohesion: 1.0
Nodes (1): Ability Picker Panel

### Community 64 - "Main Menu Screen"
Cohesion: 1.0
Nodes (1): Main Menu Screen

### Community 65 - "New Game Setup (Name/Difficulty/Seed)"
Cohesion: 1.0
Nodes (1): New Game Setup (Name/Difficulty/Seed)

### Community 66 - "Pause Menu"
Cohesion: 1.0
Nodes (1): Pause Menu

### Community 67 - "Shelter Tilemap (Walkable Compound)"
Cohesion: 1.0
Nodes (1): Shelter Tilemap (Walkable Compound)

### Community 68 - "SprintCooldown Component"
Cohesion: 1.0
Nodes (1): SprintCooldown Component

### Community 69 - "TileKind Enum"
Cohesion: 1.0
Nodes (1): TileKind Enum

### Community 70 - "ArmorDurability Component"
Cohesion: 1.0
Nodes (1): ArmorDurability Component

### Community 71 - "ItemKind Enum (Weapon/Armor/Consumable/Resource)"
Cohesion: 1.0
Nodes (1): ItemKind Enum (Weapon/Armor/Consumable/Resource)

### Community 72 - "SanityEvent Enum (Exposure Sources)"
Cohesion: 1.0
Nodes (1): SanityEvent Enum (Exposure Sources)

### Community 73 - "PendingLoad Resource"
Cohesion: 1.0
Nodes (1): PendingLoad Resource

### Community 74 - "ShelterData Struct"
Cohesion: 1.0
Nodes (1): ShelterData Struct

### Community 75 - "RaidChance Resource"
Cohesion: 1.0
Nodes (1): RaidChance Resource

### Community 76 - "ActiveRaid Resource"
Cohesion: 1.0
Nodes (1): ActiveRaid Resource

### Community 77 - "ShelterState Resource"
Cohesion: 1.0
Nodes (1): ShelterState Resource

### Community 78 - "SurvivorTask Enum"
Cohesion: 1.0
Nodes (1): SurvivorTask Enum

### Community 79 - "has_los Bresenham LOS"
Cohesion: 1.0
Nodes (1): has_los Bresenham LOS

### Community 80 - "Anomaly Component"
Cohesion: 1.0
Nodes (1): Anomaly Component

### Community 81 - "AnomalyKind Enum"
Cohesion: 1.0
Nodes (1): AnomalyKind Enum

### Community 82 - "HazardTile Component"
Cohesion: 1.0
Nodes (1): HazardTile Component

### Community 83 - "HazardKind Enum"
Cohesion: 1.0
Nodes (1): HazardKind Enum

### Community 84 - "LoreJournal Resource"
Cohesion: 1.0
Nodes (1): LoreJournal Resource

### Community 85 - "CombatRng Resource"
Cohesion: 1.0
Nodes (1): CombatRng Resource

### Community 86 - "RoomType Enum"
Cohesion: 1.0
Nodes (1): RoomType Enum

### Community 87 - "EncounterType Enum"
Cohesion: 1.0
Nodes (1): EncounterType Enum

### Community 88 - "Draw Survivor Panel"
Cohesion: 1.0
Nodes (1): Draw Survivor Panel

### Community 89 - "Draw GameLog Panel"
Cohesion: 1.0
Nodes (1): Draw GameLog Panel

### Community 90 - "Draw Inventory Panel"
Cohesion: 1.0
Nodes (1): Draw Inventory Panel

### Community 91 - "InventoryOpen Resource"
Cohesion: 1.0
Nodes (1): InventoryOpen Resource

### Community 92 - "UI Module Root"
Cohesion: 1.0
Nodes (1): UI Module Root

## Knowledge Gaps
- **231 isolated node(s):** `GabrielDialogueUiAction`, `GabrielDialogueUiChoice`, `InventoryOpen`, `InventoryUiAction`, `InventoryUiChoice` (+226 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `state.rs / AppState`** (2 nodes): `state.rs`, `AppState`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `tilemap.rs / spawn_tilemap_layer Function`** (2 nodes): `tilemap.rs`, `spawn_tilemap_layer Function`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Weighted Loot Tables (5 Quality Tiers) / Equipment Tier System (T0-T3)`** (2 nodes): `Weighted Loot Tables (5 Quality Tiers)`, `Equipment Tier System (T0-T3)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Bevy 0.18.1 (ECS Engine) / Bevy 2d Feature Flag (Eliminate 3D Pipeline)`** (2 nodes): `Bevy 0.18.1 (ECS Engine)`, `Bevy 2d Feature Flag (Eliminate 3D Pipeline)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Node Info Panel / World Node Map (Graph Not Grid)`** (2 nodes): `Node Info Panel`, `World Node Map (Graph Not Grid)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `LogColor Semantic Colors / UI Color Palette`** (2 nodes): `LogColor Semantic Colors`, `UI Color Palette`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `ItemDef Static Definition / WeaponProps (Damage/Range/Accuracy)`** (2 nodes): `ItemDef Static Definition`, `WeaponProps (Damage/Range/Accuracy)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Determinism Test Suite / WorldSeed Resource`** (2 nodes): `Determinism Test Suite`, `WorldSeed Resource`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `EnemyDef Struct / spawn_table Function`** (2 nodes): `EnemyDef Struct`, `spawn_table Function`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `DungeonFloor Struct / GabrielState Resource`** (2 nodes): `DungeonFloor Struct`, `GabrielState Resource`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `pickup_items System / pickup_lore System`** (2 nodes): `pickup_items System`, `pickup_lore System`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `JournalOpen Resource / Draw Journal Panel`** (2 nodes): `JournalOpen Resource`, `Draw Journal Panel`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Draw Perk Choice Panel / Process Perk Choice Action`** (2 nodes): `Draw Perk Choice Panel`, `Process Perk Choice Action`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Player Background System`** (1 nodes): `Player Background System`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Permadeath & Game Modes`** (1 nodes): `Permadeath & Game Modes`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Endgame Dual Paths (Sandbox vs Truth)`** (1 nodes): `Endgame Dual Paths (Sandbox vs Truth)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Status Effects (Wounded, Stunned)`** (1 nodes): `Status Effects (Wounded, Stunned)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Noise & Detection (MVP Simplified)`** (1 nodes): `Noise & Detection (MVP Simplified)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Survivors (Player Faction)`** (1 nodes): `Survivors (Player Faction)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Angelic vs Infernal Sensory Comparison`** (1 nodes): `Angelic vs Infernal Sensory Comparison`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Dungeon Theme Mixing (Zone Transitions)`** (1 nodes): `Dungeon Theme Mixing (Zone Transitions)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Phase Dependency Chain`** (1 nodes): `Phase Dependency Chain`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Dev Slice Dependency Graph (Sequential)`** (1 nodes): `Dev Slice Dependency Graph (Sequential)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `98/98 Tests Pass (Build Verified)`** (1 nodes): `98/98 Tests Pass (Build Verified)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Typography System`** (1 nodes): `Typography System`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Dungeon & Shelter Theming`** (1 nodes): `Dungeon & Shelter Theming`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Escape Cascade Priority Chain`** (1 nodes): `Escape Cascade Priority Chain`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `InGame Game Log Panel`** (1 nodes): `InGame Game Log Panel`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Ability Picker Panel`** (1 nodes): `Ability Picker Panel`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Main Menu Screen`** (1 nodes): `Main Menu Screen`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `New Game Setup (Name/Difficulty/Seed)`** (1 nodes): `New Game Setup (Name/Difficulty/Seed)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Pause Menu`** (1 nodes): `Pause Menu`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Shelter Tilemap (Walkable Compound)`** (1 nodes): `Shelter Tilemap (Walkable Compound)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `SprintCooldown Component`** (1 nodes): `SprintCooldown Component`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `TileKind Enum`** (1 nodes): `TileKind Enum`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `ArmorDurability Component`** (1 nodes): `ArmorDurability Component`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `ItemKind Enum (Weapon/Armor/Consumable/Resource)`** (1 nodes): `ItemKind Enum (Weapon/Armor/Consumable/Resource)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `SanityEvent Enum (Exposure Sources)`** (1 nodes): `SanityEvent Enum (Exposure Sources)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `PendingLoad Resource`** (1 nodes): `PendingLoad Resource`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `ShelterData Struct`** (1 nodes): `ShelterData Struct`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `RaidChance Resource`** (1 nodes): `RaidChance Resource`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `ActiveRaid Resource`** (1 nodes): `ActiveRaid Resource`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `ShelterState Resource`** (1 nodes): `ShelterState Resource`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `SurvivorTask Enum`** (1 nodes): `SurvivorTask Enum`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `has_los Bresenham LOS`** (1 nodes): `has_los Bresenham LOS`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Anomaly Component`** (1 nodes): `Anomaly Component`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `AnomalyKind Enum`** (1 nodes): `AnomalyKind Enum`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `HazardTile Component`** (1 nodes): `HazardTile Component`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `HazardKind Enum`** (1 nodes): `HazardKind Enum`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `LoreJournal Resource`** (1 nodes): `LoreJournal Resource`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `CombatRng Resource`** (1 nodes): `CombatRng Resource`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `RoomType Enum`** (1 nodes): `RoomType Enum`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `EncounterType Enum`** (1 nodes): `EncounterType Enum`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Draw Survivor Panel`** (1 nodes): `Draw Survivor Panel`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Draw GameLog Panel`** (1 nodes): `Draw GameLog Panel`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Draw Inventory Panel`** (1 nodes): `Draw Inventory Panel`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `InventoryOpen Resource`** (1 nodes): `InventoryOpen Resource`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `UI Module Root`** (1 nodes): `UI Module Root`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `RaidExposure Component (Sanity Meter)` connect `Hand-Rolled FOV (Symmetric Shadowcasting ~150 LOC) / Colony Loop Rhythm (Return→Deposit→Repair→Prepare) / Survivor Needs (Hunger/Thirst/Rest)` to `bevy_ecs_tilemap 0.18.1 (GPU Tilemap) / bevy_egui 0.39.1 (UI Framework) / egui Draw/Process Split Pattern`, `abilities.rs / calc_cover (Cover Level Calculation) / calc_range_penalty()`, `sanity.rs / apply_player_sanity_event() / apply_sanity_event()`?**
  _High betweenness centrality (0.134) - this node is a cross-community bridge._
- **Why does `check_control_loss System` connect `sanity.rs / apply_player_sanity_event() / apply_sanity_event()` to `Hand-Rolled FOV (Symmetric Shadowcasting ~150 LOC) / Colony Loop Rhythm (Return→Deposit→Repair→Prepare) / Survivor Needs (Hunger/Thirst/Rest)`?**
  _High betweenness centrality (0.072) - this node is a cross-community bridge._
- **Why does `PlayerBundle (All Player Components)` connect `abilities.rs / calc_cover (Cover Level Calculation) / calc_range_penalty()` to `Character Sheet Panel / perks.rs / apply_second_wind Function`, `Hand-Rolled FOV (Symmetric Shadowcasting ~150 LOC) / Colony Loop Rhythm (Return→Deposit→Repair→Prepare) / Survivor Needs (Hunger/Thirst/Rest)`, `camera.rs / camera_follow() / setup_camera()`?**
  _High betweenness centrality (0.044) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `RaidExposure Component (Sanity Meter)` (e.g. with `Travel System (Time/Food/Water/Danger)` and `Sanity Distortion System (UI Lies)`) actually correct?**
  _`RaidExposure Component (Sanity Meter)` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `GabrielDialogueUiAction`, `GabrielDialogueUiChoice`, `InventoryOpen` to the rest of the system?**
  _231 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `combat.rs / calc_damage Function / CheckResult Struct` be split into smaller, more focused modules?**
  _Cohesion score 0.05 - nodes in this community are weakly interconnected._
- **Should `Save/Load Interface (5 Slots) / save.rs / autosave System (On Colony Enter)` be split into smaller, more focused modules?**
  _Cohesion score 0.06 - nodes in this community are weakly interconnected._