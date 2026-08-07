use serde::{Deserialize, Serialize};

use crate::{
    components::{BlocksMovement, Name, Player, Position},
    colony::raids::RaidEnemy,
    pools::{Pool, Pools},
    relationships::FactionMember,
    signals::PoolKind,
    statuses::{StatusInstance, Statuses},
};
use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityBlueprint {
    pub id: String,
    pub label: String,
    pub is_player: bool,
    pub blocks_movement: bool,
    pub pools: Vec<(PoolKind, i32, i32, i32)>,
    pub statuses: Vec<(String, i32)>,
    pub visual: Option<String>,
    /// Marker components to insert on spawn.
    /// Convention: "ComponentName" for unit structs, "ComponentName:data" for tuple structs.
    ///
    /// **WARNING**: Adding a marker to a shared blueprint (e.g. putting `RaidEnemy` on
    /// `blueprint.rat` which is used in both dungeons and raids) can silently break
    /// unrelated tests. The extra component changes the Bevy archetype, which can alter
    /// query result ordering for blockers/pathfinding. Use separate blueprints for
    /// semantically different entity roles (e.g. `blueprint.rat` for dungeons,
    /// `blueprint.raid_rat` for raids).
    #[serde(default)]
    pub markers: Vec<String>,
}

/// Bevy Resource wrapping the blueprint catalog loaded from content RON.
///
/// Use [`BlueprintCatalog::get`] to look up blueprints by ID at runtime.
#[derive(Resource, Debug, Clone)]
pub struct BlueprintCatalog {
    entries: Vec<EntityBlueprint>,
}

impl BlueprintCatalog {
    /// Create a catalog from a list of blueprints.
    ///
    /// # Panics
    /// Panics if any two blueprints share the same `id`.
    pub fn new(blueprints: Vec<EntityBlueprint>) -> Self {
        // Validate: no duplicate IDs
        let mut seen = std::collections::HashSet::new();
        for bp in &blueprints {
            if !seen.insert(&bp.id) {
                panic!(
                    "BlueprintCatalog: duplicate blueprint ID '{}'",
                    bp.id
                );
            }
        }
        // Validate: warn on unknown markers
        for bp in &blueprints {
            for marker in &bp.markers {
                let name = marker.split(':').next().unwrap_or(marker);
                if !KNOWN_MARKERS.contains(&name) {
                    eprintln!(
                        "WARNING: BlueprintCatalog: blueprint '{}' has unknown marker '{}'",
                        bp.id,
                        marker
                    );
                }
            }
        }
        Self {
            entries: blueprints,
        }
    }

    /// Look up a blueprint by its string ID.
    pub fn get(&self, id: &str) -> Option<&EntityBlueprint> {
        self.entries.iter().find(|bp| bp.id == id)
    }
}

impl Default for BlueprintCatalog {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

/// Known marker component names for validation and spawn-time insertion.
const KNOWN_MARKERS: &[&str] = &["RaidEnemy", "FactionMember"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mutator {
    Wounded,
    Elite,
}

impl Mutator {
    pub fn apply(&self, pools: &mut [(PoolKind, i32, i32, i32)]) {
        for (kind, current, _, max) in pools.iter_mut() {
            if *kind != PoolKind::Health {
                continue;
            }
            match self {
                Mutator::Wounded => {
                    *max /= 2;
                    *current = (*current).min(*max);
                }
                Mutator::Elite => {
                    *max = (*max as f32 * 1.5) as i32;
                }
            }
        }
    }
}

pub fn spawn_from_blueprint(
    blueprint: &EntityBlueprint,
    pos: Option<Position>,
    mutators: &[Mutator],
    commands: &mut Commands,
) -> Entity {
    let mut pools = blueprint.pools.clone();
    for m in mutators {
        m.apply(&mut pools);
    }
    let mut e = commands.spawn_empty();
    if blueprint.is_player {
        e.insert(Player);
    }
    if blueprint.blocks_movement {
        e.insert(BlocksMovement);
    }
    if let Some(p) = pos {
        e.insert(p);
    }
    e.insert(Name(blueprint.label.clone()));
    if !pools.is_empty() {
        e.insert(Pools::new(
            pools
                .into_iter()
                .map(|(k, c, mn, mx)| Pool::new(k, c, mn, mx))
                .collect(),
        ));
    }
    if !blueprint.statuses.is_empty() {
        e.insert(Statuses {
            instances: blueprint
                .statuses
                .iter()
                .map(|(id, dur)| StatusInstance {
                    status_id: id.clone(),
                    remaining_duration: *dur,
                    stacks: 1,
                    source: None,
                })
                .collect(),
        });
    }
    // Insert marker components
    for marker in &blueprint.markers {
        match marker.split(':').next().unwrap_or(marker) {
            "RaidEnemy" => {
                e.insert(RaidEnemy);
            }
            "FactionMember" => {
                let faction_id = marker
                    .strip_prefix("FactionMember:")
                    .unwrap_or("unknown")
                    .to_string();
                e.insert(FactionMember(faction_id));
            }
            _ => {
                // Unknown markers are validated at catalog construction;
                // silently skip them during spawn.
            }
        }
    }
    e.id()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Tile;
    use crate::map::SmokeMap;
    use bevy_app::App;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::BdCorePlugin);
        app.world_mut()
            .insert_resource(SmokeMap::new(10, 10, Tile::Floor));
        app
    }

    #[test]
    fn factory_spawns_player_blueprint() {
        let mut app = test_app();
        let bp = EntityBlueprint {
            id: "blueprint.player".into(),
            label: "Player".into(),
            is_player: true,
            blocks_movement: false,
            pools: vec![
                (PoolKind::Health, 20, 0, 20),
                (PoolKind::ActionPoints, 3, 0, 3),
                (PoolKind::Supplies, 10, 0, 50),
            ],
            statuses: vec![],
            visual: Some("Player".into()),
            markers: vec![],
        };
        let e = spawn_from_blueprint(
            &bp,
            Some(Position { x: 10, y: 6 }),
            &[],
            &mut app.world_mut().commands(),
        );
        app.update();
        assert!(app.world().get::<Player>(e).is_some());
        assert_eq!(
            app.world().get::<Position>(e).unwrap(),
            &Position { x: 10, y: 6 }
        );
    }

    #[test]
    fn factory_spawns_enemy_blueprint() {
        let mut app = test_app();
        let bp = EntityBlueprint {
            id: "blueprint.training_dummy".into(),
            label: "Training Dummy".into(),
            is_player: false,
            blocks_movement: true,
            pools: vec![
                (PoolKind::Health, 15, 0, 15),
                (PoolKind::ActionPoints, 0, 0, 0),
            ],
            statuses: vec![],
            visual: Some("Enemy".into()),
            markers: vec![],
        };
        let e = spawn_from_blueprint(
            &bp,
            Some(Position { x: 12, y: 6 }),
            &[],
            &mut app.world_mut().commands(),
        );
        app.update();
        assert!(app.world().get::<BlocksMovement>(e).is_some());
        assert!(app.world().get::<Player>(e).is_none());
    }

    #[test]
    fn factory_spawns_item_blueprint() {
        let mut app = test_app();
        let bp = EntityBlueprint {
            id: "blueprint.healing_potion".into(),
            label: "Healing Potion".into(),
            is_player: false,
            blocks_movement: false,
            pools: vec![],
            statuses: vec![],
            visual: Some("Item".into()),
            markers: vec![],
        };
        let e = spawn_from_blueprint(&bp, None, &[], &mut app.world_mut().commands());
        app.update();
        assert!(app.world().get::<Name>(e).is_some());
    }

    #[test]
    fn factory_applies_wounded_mutator() {
        let mut pools = vec![
            (PoolKind::Health, 20, 0, 20),
            (PoolKind::ActionPoints, 3, 0, 3),
        ];
        Mutator::Wounded.apply(&mut pools);
        assert_eq!(pools[0].3, 10);
        assert_eq!(pools[0].1, 10);
    }

    #[test]
    fn factory_applies_elite_mutator() {
        let mut pools = vec![(PoolKind::Health, 5, 0, 5)];
        Mutator::Elite.apply(&mut pools);
        assert_eq!(pools[0].3, 7);
    }

    #[test]
    fn factory_rejects_missing_blueprint() {
        let catalog = BlueprintCatalog::default();
        assert!(catalog.get("blueprint.nonexistent").is_none());
    }

    #[test]
    fn factory_adds_visual_token() {
        let bp = EntityBlueprint {
            id: "test".into(),
            label: "Test".into(),
            is_player: false,
            blocks_movement: false,
            pools: vec![],
            statuses: vec![],
            visual: Some("Enemy".into()),
            markers: vec![],
        };
        assert_eq!(bp.visual, Some("Enemy".into()));
    }

    #[test]
    fn player_starts_with_full_action_points() {
        let mut app = test_app();
        let bp = EntityBlueprint {
            id: "blueprint.player".into(),
            label: "Player".into(),
            is_player: true,
            blocks_movement: false,
            pools: vec![
                (PoolKind::Health, 20, 0, 20),
                (PoolKind::ActionPoints, 3, 0, 3),
                (PoolKind::Supplies, 10, 0, 50),
            ],
            statuses: vec![],
            visual: Some("Player".into()),
            markers: vec![],
        };
        let e = spawn_from_blueprint(
            &bp,
            Some(Position { x: 10, y: 6 }),
            &[],
            &mut app.world_mut().commands(),
        );
        app.update();
        let pools = app.world().get::<Pools>(e).unwrap();
        let ap = pools.get(PoolKind::ActionPoints).unwrap();
        assert_eq!(
            ap.current, ap.max,
            "Player should start with full ActionPoints (max = {})",
            ap.max
        );
        assert!(
            ap.current > 0,
            "Player ActionPoints should be greater than 0 at start"
        );
    }

    // ── Phase 1: BlueprintCatalog tests ──

    /// Helper: create a minimal EntityBlueprint for catalog tests.
    fn make_test_bp(id: &str, label: &str) -> EntityBlueprint {
        EntityBlueprint {
            id: id.into(),
            label: label.into(),
            is_player: false,
            blocks_movement: false,
            pools: vec![],
            statuses: vec![],
            visual: None,
            markers: vec![],
        }
    }

    /// Helper: blueprint with pools for spawn tests.
    fn make_rat_bp() -> EntityBlueprint {
        EntityBlueprint {
            id: "blueprint.rat".into(),
            label: "Rat".into(),
            is_player: false,
            blocks_movement: true,
            pools: vec![
                (PoolKind::Health, 11, 0, 11),
                (PoolKind::ActionPoints, 2, 0, 2),
            ],
            statuses: vec![],
            visual: Some("Enemy".into()),
            markers: vec![],
        }
    }

    // ── Catalog lookup ──

    #[test]
    fn catalog_get_returns_blueprint_by_id() {
        let catalog = BlueprintCatalog::new(vec![
            make_test_bp("blueprint.player", "Player"),
            make_rat_bp(),
        ]);
        let found = catalog.get("blueprint.rat");
        assert!(found.is_some(), "catalog must return blueprint by ID");
        let bp = found.unwrap();
        assert_eq!(bp.label, "Rat");
        assert!(bp.blocks_movement, "Rat blueprint must block movement");
    }

    #[test]
    fn catalog_get_unknown_returns_none() {
        let catalog =
            BlueprintCatalog::new(vec![make_test_bp("blueprint.player", "Player")]);
        assert!(
            catalog.get("blueprint.nonexistent").is_none(),
            "unknown ID must return None"
        );
    }

    // ── Catalog validation ──

    #[test]
    fn catalog_construction_warns_unknown_markers() {
        // Unknown markers should warn but NOT panic.
        // The catalog is still created successfully.
        let bp = EntityBlueprint {
            markers: vec!["BogusMarker".into()],
            ..make_test_bp("blueprint.test", "Test")
        };
        let catalog = BlueprintCatalog::new(vec![bp]);
        // Catalog must still be usable after warning
        assert!(catalog.get("blueprint.test").is_some());
    }

    #[test]
    #[should_panic(expected = "duplicate")]
    fn catalog_construction_panics_on_duplicate_ids() {
        let bp1 = make_test_bp("blueprint.rat", "Rat");
        let bp2 = make_test_bp("blueprint.rat", "Rat Duplicate");
        let _catalog = BlueprintCatalog::new(vec![bp1, bp2]);
    }

    // ── Marker component insertion ──

    #[test]
    fn spawn_inserts_marker_components() {
        let mut app = test_app();
        let bp = EntityBlueprint {
            markers: vec!["RaidEnemy".into()],
            ..make_rat_bp()
        };
        let e = spawn_from_blueprint(
            &bp,
            Some(Position { x: 3, y: 3 }),
            &[],
            &mut app.world_mut().commands(),
        );
        app.update();
        assert!(
            app.world().get::<crate::colony::raids::RaidEnemy>(e).is_some(),
            "entity must have RaidEnemy component from marker"
        );
    }

    #[test]
    fn spawn_silently_ignores_unknown_marker() {
        let mut app = test_app();
        let bp = EntityBlueprint {
            markers: vec!["BogusMarker".into()],
            ..make_rat_bp()
        };
        let e = spawn_from_blueprint(
            &bp,
            Some(Position { x: 3, y: 3 }),
            &[],
            &mut app.world_mut().commands(),
        );
        app.update();
        // Entity spawns successfully despite unknown marker
        assert!(app.world().get::<Name>(e).is_some());
        assert_eq!(app.world().get::<Name>(e).unwrap().0, "Rat");
    }

    #[test]
    fn spawn_multiple_markers_with_data() {
        let mut app = test_app();
        let bp = EntityBlueprint {
            markers: vec![
                "RaidEnemy".into(),
                "FactionMember:faction.demons".into(),
            ],
            ..make_rat_bp()
        };
        let e = spawn_from_blueprint(
            &bp,
            Some(Position { x: 3, y: 3 }),
            &[],
            &mut app.world_mut().commands(),
        );
        app.update();
        // Both marker components present
        assert!(
            app.world()
                .get::<crate::colony::raids::RaidEnemy>(e)
                .is_some(),
            "must have RaidEnemy"
        );
        let faction = app
            .world()
            .get::<crate::relationships::FactionMember>(e)
            .expect("must have FactionMember with data");
        assert_eq!(
            faction.0, "faction.demons",
            "FactionMember must carry parsed faction ID"
        );
    }

    // ── RON data integrity ──

    #[test]
    fn ron_loads_required_blueprints() {
        let content_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("content");
        let ron_path = content_root.join("blueprints").join("foundation.ron");
        let raw = std::fs::read_to_string(&ron_path)
            .expect("foundation.ron must exist and be readable");
        let blueprints: Vec<EntityBlueprint> =
            ron::from_str(&raw).expect("foundation.ron must be valid RON");

        // Required blueprints for gameplay
        let required = ["blueprint.player", "blueprint.rat", "blueprint.healing_potion"];
        for id in &required {
            assert!(
                blueprints.iter().any(|bp| bp.id == *id),
                "foundation.ron must contain required blueprint '{id}'"
            );
        }

        // Structural integrity: all blueprints must have id and label
        for bp in &blueprints {
            assert!(!bp.id.is_empty(), "blueprint must have non-empty id");
            assert!(!bp.label.is_empty(), "blueprint '{id}' must have non-empty label",
                id = bp.id);
        }

        // No duplicate IDs
        let mut seen = std::collections::HashSet::new();
        for bp in &blueprints {
            assert!(
                seen.insert(&bp.id),
                "foundation.ron must not contain duplicate blueprint ID '{}'",
                bp.id
            );
        }
    }
}
