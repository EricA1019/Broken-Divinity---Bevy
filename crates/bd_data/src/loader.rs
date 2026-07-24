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
    let blueprints = load_items!("blueprints", bd_core::factory::EntityBlueprint);

    let bundle = FoundationContent {
        dungeons,
        items,
        skills,
        factions,
        actions,
        blueprints,
    };
    validate_foundation_content(&bundle)?;
    Ok(bundle)
}

fn validate_foundation_content(
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
    for entry in &content.blueprints {
        add_id(&entry.id, "blueprints/foundation.ron")?;
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

    for skill in &content.skills {
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bd_core::content::FactionDefinition;

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
            hostility: "neutral".into(),
        });
        validate_foundation_content(&content).unwrap();
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
}
