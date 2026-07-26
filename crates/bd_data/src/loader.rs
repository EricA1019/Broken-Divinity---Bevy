use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    #[error("IO error in {file}: {message}")]
    Io { file: String, message: String },
    #[error("RON parse error in {file}: {message}")]
    Ron { file: String, message: String },
    #[error("Content validation error in {file}: {message}")]
    Validation { file: String, message: String },
}

/// A RON content file with source path for error reporting.
#[derive(Debug)]
pub struct ContentFile<T> {
    pub path: String,
    pub items: Vec<T>,
}

/// Load a RON file from disk and deserialize into Vec<T>.
pub fn load_ron<T: serde::de::DeserializeOwned>(path: &Path) -> Result<ContentFile<T>, LoadError> {
    let content = std::fs::read_to_string(path).map_err(|e| LoadError::Io {
        file: path.to_string_lossy().into_owned(),
        message: e.to_string(),
    })?;
    let items: Vec<T> = ron::from_str(&content).map_err(|e| LoadError::Ron {
        file: path.to_string_lossy().into_owned(),
        message: e.to_string(),
    })?;
    Ok(ContentFile {
        path: path.to_string_lossy().into_owned(),
        items,
    })
}

/// Load and validate the complete foundation content bundle.
pub fn load_foundation_content(
    content_dir: &Path,
) -> Result<bd_core::content::FoundationContent, LoadError> {
    use bd_core::content::FoundationContent;
    macro_rules! load_items {
        ($folder:literal, $ty:ty) => {
            load_ron::<$ty>(&content_dir.join($folder).join("foundation.ron"))?.items
        };
    }

    let dungeons = load_items!("dungeons", bd_core::content::DungeonDefinition);
    let items = load_items!("items", bd_core::content::ItemDefinition);
    let skills = load_items!("skills", bd_core::content::SkillDefinition);
    let factions = load_items!("factions", bd_core::content::FactionDefinition);
    let actions = load_items!("actions", bd_core::content::ActionReference);
    let stations = load_items!("stations", bd_core::colony::stations::StationBlueprint);
    let blueprints = load_items!("blueprints", bd_core::factory::EntityBlueprint);

    let bundle = FoundationContent {
        dungeons,
        items,
        skills,
        factions,
        actions,
        stations,
        blueprints,
    };
    validate_foundation_content(&bundle)?;
    Ok(bundle)
}

pub fn validate_foundation_content(
    content: &bd_core::content::FoundationContent,
) -> Result<(), LoadError> {
    use bd_core::ids::ContentId;
    use std::collections::HashSet;

    let mut ids = HashSet::new();
    let mut add_id = |id: &str, file: &str| -> Result<(), LoadError> {
        ContentId::parse(id).map_err(|e| LoadError::Validation {
            file: file.into(),
            message: format!("record '{id}': {e}"),
        })?;
        if !ids.insert(id.to_owned()) {
            return Err(LoadError::Validation {
                file: file.into(),
                message: format!("duplicate content ID '{id}'"),
            });
        }
        Ok(())
    };

    for entry in &content.dungeons {
        add_id(&entry.id, "dungeons/foundation.ron")?;
    }
    for entry in &content.items {
        add_id(&entry.id, "items/foundation.ron")?;
    }
    for entry in &content.skills {
        add_id(&entry.id, "skills/foundation.ron")?;
    }
    for entry in &content.factions {
        add_id(&entry.id, "factions/foundation.ron")?;
    }
    for entry in &content.actions {
        add_id(&entry.id, "actions/foundation.ron")?;
    }
    for entry in &content.stations {
        add_id(&entry.id, "stations/foundation.ron")?;
    }
    for entry in &content.blueprints {
        add_id(&entry.id, "blueprints/foundation.ron")?;
    }

    let mut station_types = HashSet::new();
    for station in &content.stations {
        if !station_types.insert(station.station_type) {
            return Err(LoadError::Validation {
                file: "stations/foundation.ron".into(),
                message: format!("duplicate station type {:?}", station.station_type),
            });
        }
        if station.label.trim().is_empty() || station.description.trim().is_empty() {
            return Err(LoadError::Validation {
                file: "stations/foundation.ron".into(),
                message: format!("station '{}' requires a label and description", station.id),
            });
        }
        if station.build_cost_supplies < 0 {
            return Err(LoadError::Validation {
                file: "stations/foundation.ron".into(),
                message: format!("station '{}' has a negative build cost", station.id),
            });
        }
        match station.effect {
            bd_core::colony::stations::StationEffect::Produce { amount, .. }
            | bd_core::colony::stations::StationEffect::RestoreWorkerMood { amount }
                if amount <= 0 =>
            {
                return Err(LoadError::Validation {
                    file: "stations/foundation.ron".into(),
                    message: format!("station '{}' effect amount must be positive", station.id),
                });
            }
            bd_core::colony::stations::StationEffect::Disabled
                if station.buildable || station.unavailable_reason.is_none() =>
            {
                return Err(LoadError::Validation {
                    file: "stations/foundation.ron".into(),
                    message: format!(
                        "disabled station '{}' must be unavailable with a visible reason",
                        station.id
                    ),
                });
            }
            _ => {}
        }
    }

    let mut active_glyphs = std::collections::HashMap::<char, String>::from([
        ('@', "Player".into()),
        ('i', "Survivor Idle".into()),
        ('e', "Survivor EnRoute".into()),
        ('*', "Survivor Working".into()),
        ('x', "Survivor Blocked".into()),
        ('r', "Survivor Resting".into()),
        ('d', "Survivor Defending".into()),
        ('T', "Trees".into()),
        ('~', "Water Source".into()),
        ('P', "Wild Plants".into()),
        ('>', "Shelter gate".into()),
    ]);
    for station in &content.stations {
        for (state, glyph) in [
            ("unstaffed", station.glyph),
            ("staffed", station.staffed_glyph),
        ] {
            let identity = format!("{} ({state})", station.id);
            if let Some(existing) = active_glyphs.insert(glyph, identity.clone()) {
                return Err(LoadError::Validation {
                    file: "stations/foundation.ron".into(),
                    message: format!(
                        "ambiguous Foundation glyph '{glyph}' is shared by {existing} and {identity}"
                    ),
                });
            }
        }
    }

    let required = [
        "dungeon.foundation",
        "item.healing_potion",
        "skill.melee",
        "skill.ranged",
        "skill.repair",
        "skill.medicine",
        "faction.placeholder_a",
        "faction.placeholder_b",
        "station.stove",
        "station.altar",
        "station.workshop",
        "station.bed",
        "station.storage",
        "blueprint.player",
        "blueprint.rat",
        "blueprint.healing_potion",
    ];
    for id in required {
        if !ids.contains(id) {
            return Err(LoadError::Validation {
                file: "foundation bundle".into(),
                message: format!("missing required record '{id}'"),
            });
        }
    }

    let blueprint_ids: HashSet<_> = content.blueprints.iter().map(|e| e.id.as_str()).collect();
    let item_ids: HashSet<_> = content.items.iter().map(|e| e.id.as_str()).collect();
    let action_ids: HashSet<_> = content.actions.iter().map(|e| e.id.as_str()).collect();
    let faction_ids: HashSet<_> = content.factions.iter().map(|e| e.id.as_str()).collect();
    const FOUNDATION_VIRTUES: &[&str] = &[
        "virtue.temperance",
        "virtue.justice",
        "virtue.prudence",
        "virtue.fortitude",
        "virtue.thumos",
        "virtue.metis",
        "virtue.kleos",
    ];

    for skill in &content.skills {
        if !FOUNDATION_VIRTUES.contains(&skill.governing_virtue.as_str()) {
            return Err(LoadError::Validation {
                file: "skills/foundation.ron".into(),
                message: format!(
                    "record '{}' references unknown governing virtue '{}'",
                    skill.id, skill.governing_virtue
                ),
            });
        }
        let Some(action) = content.actions.iter().find(|action| {
            action.id == skill.action_id && action.skill_id.as_deref() == Some(skill.id.as_str())
        }) else {
            return Err(LoadError::Validation {
                file: "actions/foundation.ron".into(),
                message: format!("skill '{}' has no matching action metadata", skill.id),
            });
        };
        if action.skill_gain <= 0 || action.virtue_gain <= 0 || action.virtue_expression.is_none() {
            return Err(LoadError::Validation {
                file: "actions/foundation.ron".into(),
                message: format!("action '{}' has incomplete progression metadata", action.id),
            });
        }
        if !action
            .virtue_expression
            .as_deref()
            .is_some_and(|virtue| FOUNDATION_VIRTUES.contains(&virtue))
        {
            return Err(LoadError::Validation {
                file: "actions/foundation.ron".into(),
                message: format!(
                    "action '{}' references unknown virtue '{}'",
                    action.id,
                    action.virtue_expression.as_deref().unwrap_or("<missing>")
                ),
            });
        }
    }

    for item in &content.items {
        if !blueprint_ids.contains(item.blueprint_id.as_str()) {
            return Err(LoadError::Validation {
                file: "items/foundation.ron".into(),
                message: format!(
                    "record '{}' references missing blueprint '{}'",
                    item.id, item.blueprint_id
                ),
            });
        }
        if item.usable && item.healing_amount.unwrap_or(0) <= 0 {
            return Err(LoadError::Validation {
                file: "items/foundation.ron".into(),
                message: format!("record '{}' must define positive healing_amount", item.id),
            });
        }
    }
    for skill in &content.skills {
        if !action_ids.contains(skill.action_id.as_str()) {
            return Err(LoadError::Validation {
                file: "skills/foundation.ron".into(),
                message: format!(
                    "record '{}' references missing action '{}'",
                    skill.id, skill.action_id
                ),
            });
        }
        if skill.progression_rate == 0 {
            return Err(LoadError::Validation {
                file: "skills/foundation.ron".into(),
                message: format!(
                    "record '{}' must have a positive progression_rate",
                    skill.id
                ),
            });
        }
    }
    for dungeon in &content.dungeons {
        if dungeon.width <= 0
            || dungeon.height <= 0
            || dungeon.tiles.len() != (dungeon.width * dungeon.height) as usize
        {
            return Err(LoadError::Validation {
                file: "dungeons/foundation.ron".into(),
                message: format!(
                    "record '{}' has invalid dimensions or tile count",
                    dungeon.id
                ),
            });
        }
        let in_bounds = |p: bd_core::components::Position| {
            p.x >= 0 && p.x < dungeon.width && p.y >= 0 && p.y < dungeon.height
        };
        if !in_bounds(dungeon.entrance) || !in_bounds(dungeon.extraction) {
            return Err(LoadError::Validation {
                file: "dungeons/foundation.ron".into(),
                message: format!(
                    "record '{}' has an out-of-bounds entrance or extraction",
                    dungeon.id
                ),
            });
        }
        let tile_at = |position: bd_core::components::Position| {
            dungeon.tiles[(position.y * dungeon.width + position.x) as usize]
        };
        if !tile_at(dungeon.entrance).is_walkable() {
            return Err(LoadError::Validation {
                file: "dungeons/foundation.ron".into(),
                message: format!("record '{}' entrance must be walkable", dungeon.id),
            });
        }
        if !tile_at(dungeon.extraction).is_walkable() {
            return Err(LoadError::Validation {
                file: "dungeons/foundation.ron".into(),
                message: format!("record '{}' extraction must be walkable", dungeon.id),
            });
        }
        if !positions_connected(dungeon, dungeon.entrance, dungeon.extraction) {
            return Err(LoadError::Validation {
                file: "dungeons/foundation.ron".into(),
                message: format!(
                    "record '{}' extraction is not reachable from entrance",
                    dungeon.id
                ),
            });
        }
        let mut occupied = HashSet::new();
        for placement in dungeon
            .enemy_placements
            .iter()
            .chain(dungeon.item_placements.iter())
        {
            if !in_bounds(placement.position) {
                return Err(LoadError::Validation {
                    file: "dungeons/foundation.ron".into(),
                    message: format!(
                        "record '{}' has out-of-bounds placement '{}'",
                        dungeon.id, placement.content_id
                    ),
                });
            }
            if !tile_at(placement.position).is_walkable() {
                return Err(LoadError::Validation {
                    file: "dungeons/foundation.ron".into(),
                    message: format!(
                        "record '{}' placement '{}' must be on a walkable tile",
                        dungeon.id, placement.content_id
                    ),
                });
            }
            if placement.position == dungeon.entrance
                || placement.position == dungeon.extraction
                || !occupied.insert(placement.position)
            {
                return Err(LoadError::Validation {
                    file: "dungeons/foundation.ron".into(),
                    message: format!(
                        "record '{}' has illegal placement overlap at ({}, {})",
                        dungeon.id, placement.position.x, placement.position.y
                    ),
                });
            }
            if let Some(faction_id) = placement.faction_id.as_deref()
                && !faction_ids.contains(faction_id)
            {
                return Err(LoadError::Validation {
                    file: "dungeons/foundation.ron".into(),
                    message: format!(
                        "record '{}' references missing faction '{}'",
                        dungeon.id, faction_id
                    ),
                });
            }
        }
        for placement in &dungeon.item_placements {
            if !item_ids.contains(placement.content_id.as_str()) {
                return Err(LoadError::Validation {
                    file: "dungeons/foundation.ron".into(),
                    message: format!(
                        "record '{}' references missing item '{}'",
                        dungeon.id, placement.content_id
                    ),
                });
            }
        }
        for placement in &dungeon.enemy_placements {
            if !blueprint_ids.contains(placement.content_id.as_str()) {
                return Err(LoadError::Validation {
                    file: "dungeons/foundation.ron".into(),
                    message: format!(
                        "record '{}' references missing enemy blueprint '{}'",
                        dungeon.id, placement.content_id
                    ),
                });
            }
        }
    }

    let Some(player) = content
        .blueprints
        .iter()
        .find(|blueprint| blueprint.id == "blueprint.player" && blueprint.is_player)
    else {
        return Err(LoadError::Validation {
            file: "blueprints/foundation.ron".into(),
            message: "required player blueprint is missing or not marked as player".into(),
        });
    };
    for virtue in bd_core::virtues::ALL_VIRTUES {
        if !player.pools.iter().any(|(kind, _, _, _)| kind == virtue) {
            return Err(LoadError::Validation {
                file: "blueprints/foundation.ron".into(),
                message: format!(
                    "record '{}' is missing required virtue state {virtue:?}",
                    player.id
                ),
            });
        }
    }
    Ok(())
}

fn positions_connected(
    dungeon: &bd_core::content::DungeonDefinition,
    start: bd_core::components::Position,
    goal: bd_core::components::Position,
) -> bool {
    use bd_core::components::Position;
    use std::collections::{HashSet, VecDeque};

    let mut frontier = VecDeque::from([start]);
    let mut visited = HashSet::from([start]);
    while let Some(position) = frontier.pop_front() {
        if position == goal {
            return true;
        }
        for next in [
            Position {
                x: position.x,
                y: position.y - 1,
            },
            Position {
                x: position.x,
                y: position.y + 1,
            },
            Position {
                x: position.x - 1,
                y: position.y,
            },
            Position {
                x: position.x + 1,
                y: position.y,
            },
        ] {
            let in_bounds =
                next.x >= 0 && next.x < dungeon.width && next.y >= 0 && next.y < dungeon.height;
            if !in_bounds || !visited.insert(next) {
                continue;
            }
            let tile = dungeon.tiles[(next.y * dungeon.width + next.x) as usize];
            if tile.is_walkable() {
                frontier.push_back(next);
            }
        }
    }
    false
}

pub fn validate_runtime_action_links(
    content: &bd_core::content::FoundationContent,
    mut is_registered: impl FnMut(&str) -> bool,
) -> Result<(), LoadError> {
    for action in &content.actions {
        if !is_registered(&action.id) {
            return Err(LoadError::Validation {
                file: "actions/foundation.ron".into(),
                message: format!(
                    "content action '{}' is not registered by the Foundation runtime",
                    action.id
                ),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bd_core::{
        components::{Position, Tile},
        content::{ActionReference, FactionDefinition},
        signals::PoolKind,
    };

    fn content_root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("content")
    }

    #[test]
    fn foundation_content_loads() {
        let content = load_foundation_content(&content_root()).unwrap();
        assert_eq!(content.dungeons.len(), 1);
        assert_eq!(content.items.len(), 1);
        assert_eq!(content.factions.len(), 2);
        assert_eq!(content.factions[0].label, "The Ashbound");
        assert_eq!(content.factions[1].label, "The Wayfarers");
    }

    #[test]
    fn faction_display_name_is_content_driven() {
        let mut content = load_foundation_content(&content_root()).unwrap();
        content.factions[0].label = "Renamed Placeholder".into();
        validate_foundation_content(&content).unwrap();
        assert_eq!(content.factions[0].label, "Renamed Placeholder");
    }

    #[test]
    fn third_faction_loads_without_a_rust_branch() {
        let mut content = load_foundation_content(&content_root()).unwrap();
        content.factions.push(FactionDefinition {
            id: "faction.placeholder_c".into(),
            label: "The Third Placeholder".into(),
            identity_key: "placeholder_c".into(),
            disposition: bd_core::content::FoundationDisposition::Neutral,
        });
        validate_foundation_content(&content).unwrap();
    }

    #[test]
    fn sixth_station_loads_without_a_rust_branch() {
        let mut content = load_foundation_content(&content_root()).unwrap();
        content
            .stations
            .push(bd_core::colony::stations::StationBlueprint {
                id: "station.test_extension".into(),
                station_type: bd_core::colony::stations::StationType::Custom(900),
                label: "Test Extension".into(),
                build_cost_supplies: 1,
                description: "Validation-only extension record.".into(),
                glyph: 'q',
                staffed_glyph: 'Q',
                effect: bd_core::colony::stations::StationEffect::Produce {
                    kind: PoolKind::Materials,
                    amount: 1,
                },
                staffing_required: true,
                buildable: true,
                unavailable_reason: None,
            });

        validate_foundation_content(&content).unwrap();
    }

    #[test]
    fn station_fallback_collision_names_both_content_records() {
        let mut content = load_foundation_content(&content_root()).unwrap();
        content.stations[1].glyph = content.stations[0].glyph;

        let error = validate_foundation_content(&content)
            .unwrap_err()
            .to_string();

        assert!(error.contains("station.stove"), "{error}");
        assert!(error.contains("station.altar"), "{error}");
    }

    #[test]
    fn station_and_resource_fallback_collision_is_rejected() {
        let mut content = load_foundation_content(&content_root()).unwrap();
        let workshop = content
            .stations
            .iter_mut()
            .find(|station| station.id == "station.workshop")
            .expect("Foundation Workshop must exist");
        workshop.staffed_glyph = '~';

        let error = validate_foundation_content(&content)
            .unwrap_err()
            .to_string();

        assert!(error.contains("station.workshop"), "{error}");
        assert!(error.contains("Water Source"), "{error}");
    }

    #[test]
    fn duplicate_content_ids_fail() {
        let mut content = load_foundation_content(&content_root()).unwrap();
        content.items.push(content.items[0].clone());
        let error = validate_foundation_content(&content).unwrap_err();
        assert!(error.to_string().contains("duplicate content ID"));
    }

    #[test]
    fn missing_content_reference_fails() {
        let mut content = load_foundation_content(&content_root()).unwrap();
        content.items[0].blueprint_id = "blueprint.missing".into();
        let error = validate_foundation_content(&content).unwrap_err();
        assert!(error.to_string().contains("blueprint.missing"));
    }

    #[test]
    fn malformed_foundation_ron_reports_file() {
        let path = std::env::temp_dir().join("bd-malformed-foundation.ron");
        std::fs::write(&path, "[not valid ron").unwrap();
        let error = load_ron::<bd_core::content::ItemDefinition>(&path).unwrap_err();
        assert!(error.to_string().contains("bd-malformed-foundation.ron"));
        let _ = std::fs::remove_file(path);
    }

    fn tile_index(dungeon: &bd_core::content::DungeonDefinition, position: Position) -> usize {
        (position.y * dungeon.width + position.x) as usize
    }

    #[test]
    fn dungeon_entrance_must_be_walkable() {
        let mut content = load_foundation_content(&content_root()).unwrap();
        let dungeon = &mut content.dungeons[0];
        let index = tile_index(dungeon, dungeon.entrance);
        dungeon.tiles[index] = Tile::Wall;

        let error = validate_foundation_content(&content).unwrap_err();
        assert!(error.to_string().contains("entrance"));
        assert!(error.to_string().contains("walkable"));
    }

    #[test]
    fn dungeon_extraction_must_be_walkable() {
        let mut content = load_foundation_content(&content_root()).unwrap();
        let dungeon = &mut content.dungeons[0];
        let index = tile_index(dungeon, dungeon.extraction);
        dungeon.tiles[index] = Tile::Wall;

        let error = validate_foundation_content(&content).unwrap_err();
        assert!(error.to_string().contains("extraction"));
        assert!(error.to_string().contains("walkable"));
    }

    #[test]
    fn dungeon_extraction_must_be_reachable() {
        let mut content = load_foundation_content(&content_root()).unwrap();
        let dungeon = &mut content.dungeons[0];
        for position in [Position { x: 9, y: 6 }, Position { x: 10, y: 5 }] {
            let index = tile_index(dungeon, position);
            dungeon.tiles[index] = Tile::Wall;
        }

        let error = validate_foundation_content(&content).unwrap_err();
        assert!(error.to_string().contains("not reachable"));
    }

    #[test]
    fn placement_must_be_on_walkable_tile() {
        let mut content = load_foundation_content(&content_root()).unwrap();
        let dungeon = &mut content.dungeons[0];
        dungeon.enemy_placements[0].position = Position { x: 0, y: 0 };

        let error = validate_foundation_content(&content).unwrap_err();
        assert!(error.to_string().contains("placement"));
        assert!(error.to_string().contains("walkable"));
    }

    #[test]
    fn placements_must_not_overlap_illegally() {
        let mut content = load_foundation_content(&content_root()).unwrap();
        content.dungeons[0].item_placements[0].position =
            content.dungeons[0].enemy_placements[0].position;

        let error = validate_foundation_content(&content).unwrap_err();
        assert!(error.to_string().contains("overlap"));
    }

    #[test]
    fn virtue_reference_must_exist() {
        let mut content = load_foundation_content(&content_root()).unwrap();
        content.skills[0].governing_virtue = "virtue.unknown".into();

        let error = validate_foundation_content(&content).unwrap_err();
        assert!(error.to_string().contains("virtue.unknown"));
    }

    #[test]
    fn faction_disposition_must_be_valid() {
        let error = ron::from_str::<FactionDefinition>(
            r#"FactionDefinition(
                id: "faction.invalid",
                label: "Invalid",
                identity_key: "invalid",
                disposition: Friendly,
            )"#,
        )
        .unwrap_err();
        assert!(error.to_string().contains("Friendly"));
    }

    #[test]
    fn content_action_must_be_registered() {
        let mut content = load_foundation_content(&content_root()).unwrap();
        content.actions.push(ActionReference {
            id: "ability.unregistered".into(),
            label: "Unregistered".into(),
            skill_id: None,
            skill_gain: 0,
            virtue_expression: None,
            virtue_gain: 0,
        });

        let mut app = bevy_app::App::new();
        app.add_plugins(bd_core::BdFoundationPlugin);
        let error = validate_runtime_action_links(&content, |action_id| {
            bd_core::foundation_action_is_registered(app.world(), action_id)
        })
        .unwrap_err();
        assert!(error.to_string().contains("ability.unregistered"));
    }

    #[test]
    fn required_player_virtue_state_is_validated() {
        let mut content = load_foundation_content(&content_root()).unwrap();
        let player = content
            .blueprints
            .iter_mut()
            .find(|blueprint| blueprint.id == "blueprint.player")
            .unwrap();
        player
            .pools
            .retain(|(kind, _, _, _)| *kind != PoolKind::Justice);

        let error = validate_foundation_content(&content).unwrap_err();
        assert!(error.to_string().contains("Justice"));
    }
}
