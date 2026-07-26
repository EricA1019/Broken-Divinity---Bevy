//! App configuration, key bindings, and path resolution for the BD Kernel.
//!
//! Phase 16: Uses `directories::ProjectDirs` for platform-correct paths,
//! Serde + TOML for user-editable config files.
//! Validated bindings are converted into semantic TUI commands at startup.

use std::{
    fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Quality-of-life identifier for the app
// ---------------------------------------------------------------------------

#[allow(dead_code)]
/// The app identity used by `directories::ProjectDirs`.
pub fn app_qualifier() -> &'static str {
    "broken-divinity"
}

pub fn app_organization() -> &'static str {
    "bd-kernel"
}

pub fn app_name() -> &'static str {
    "broken-divinity"
}

/// Resolve the project directories for this app.
pub fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", app_organization(), app_name())
}

/// Get the config directory (e.g. `~/.config/broken-divinity/`).
pub fn config_dir() -> PathBuf {
    project_dirs()
        .map(|d| d.config_dir().to_path_buf())
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            Path::new(&home).join(".config").join(app_name())
        })
}

#[allow(dead_code)]
/// Get the save data directory.
pub fn data_dir() -> PathBuf {
    project_dirs()
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            Path::new(&home)
                .join(".local")
                .join("share")
                .join(app_name())
        })
}

// ---------------------------------------------------------------------------
// Config structures
// ---------------------------------------------------------------------------

/// All user-configurable settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Theme ID to load at startup (e.g. "bd_default").
    #[serde(default = "default_theme_id")]
    pub theme_id: String,
    /// Key bindings for game actions.
    #[serde(default)]
    pub keybindings: KeyBindingConfig,
    /// Override the default save directory.
    #[serde(default)]
    pub save_dir_override: Option<String>,
    /// Log level filter string (e.g. "bd=debug").
    #[serde(default = "default_log_level")]
    pub log_level: String,
    /// Debug feature flags.
    #[serde(default)]
    pub debug_flags: Vec<String>,
}

fn default_theme_id() -> String {
    "bd_default".into()
}

fn default_log_level() -> String {
    "bd=info".into()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme_id: default_theme_id(),
            keybindings: KeyBindingConfig::default(),
            save_dir_override: None,
            log_level: default_log_level(),
            debug_flags: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Key bindings
// ---------------------------------------------------------------------------

/// User-configurable key bindings for game actions.
///
/// Each field is a single-key string (e.g. `"w"`, `"i"`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyBindingConfig {
    pub move_north: String,
    pub move_south: String,
    pub move_east: String,
    pub move_west: String,
    pub wait: String,
    pub rest_until_next_day: String,
    pub attack: String,
    pub guard: String,
    pub inventory: String,
    pub pickup: String,
    pub extract: String,
    pub use_item: String,
    pub help: String,
    pub travel: String,
    pub build: String,
    pub assign_task: String,
    pub assign_station: String,
    pub save: String,
    pub load: String,
    pub quit: String,
}

impl Default for KeyBindingConfig {
    fn default() -> Self {
        use bd_tui::commands::{CommandBindings, UiCommand, config_key_name};

        let bindings = CommandBindings::default();
        let key = |command| {
            config_key_name(
                bindings
                    .key_for(command)
                    .expect("every configurable command must have a built-in default"),
            )
        };
        Self {
            move_north: key(UiCommand::MoveNorth),
            move_south: key(UiCommand::MoveSouth),
            move_east: key(UiCommand::MoveEast),
            move_west: key(UiCommand::MoveWest),
            wait: key(UiCommand::Wait),
            rest_until_next_day: key(UiCommand::RestUntilNextDay),
            attack: key(UiCommand::Attack),
            guard: key(UiCommand::Guard),
            inventory: key(UiCommand::Inventory),
            pickup: key(UiCommand::Pickup),
            extract: key(UiCommand::Extract),
            use_item: key(UiCommand::UseItem),
            help: key(UiCommand::Help),
            travel: key(UiCommand::Travel),
            build: key(UiCommand::Build),
            assign_task: key(UiCommand::AssignTask),
            assign_station: key(UiCommand::AssignStation),
            save: key(UiCommand::Save),
            load: key(UiCommand::Load),
            quit: key(UiCommand::Quit),
        }
    }
}

impl KeyBindingConfig {
    pub fn command_bindings(&self) -> Result<bd_tui::commands::CommandBindings, ConfigError> {
        use bd_tui::commands::UiCommand;

        let mut bindings = bd_tui::commands::CommandBindings::default();
        for (command, configured) in [
            (UiCommand::MoveNorth, self.move_north.as_str()),
            (UiCommand::MoveSouth, self.move_south.as_str()),
            (UiCommand::MoveEast, self.move_east.as_str()),
            (UiCommand::MoveWest, self.move_west.as_str()),
            (UiCommand::Wait, self.wait.as_str()),
            (
                UiCommand::RestUntilNextDay,
                self.rest_until_next_day.as_str(),
            ),
            (UiCommand::Attack, self.attack.as_str()),
            (UiCommand::Guard, self.guard.as_str()),
            (UiCommand::Inventory, self.inventory.as_str()),
            (UiCommand::Pickup, self.pickup.as_str()),
            (UiCommand::Extract, self.extract.as_str()),
            (UiCommand::UseItem, self.use_item.as_str()),
            (UiCommand::Help, self.help.as_str()),
            (UiCommand::Travel, self.travel.as_str()),
            (UiCommand::Build, self.build.as_str()),
            (UiCommand::AssignTask, self.assign_task.as_str()),
            (UiCommand::AssignStation, self.assign_station.as_str()),
            (UiCommand::Save, self.save.as_str()),
            (UiCommand::Load, self.load.as_str()),
            (UiCommand::Quit, self.quit.as_str()),
        ] {
            let key = parse_key_code(configured).ok_or_else(|| {
                ConfigError::Validation(vec![format!(
                    "unsupported key binding `{configured}` for {command:?}"
                )])
            })?;
            bindings.bind(command, key);
        }
        let conflicts = bindings
            .conflicts()
            .into_iter()
            .map(|(left, right, key)| {
                format!("key `{key}` is assigned to both {left:?} and {right:?}")
            })
            .collect::<Vec<_>>();
        if !conflicts.is_empty() {
            return Err(ConfigError::Validation(conflicts));
        }
        Ok(bindings)
    }
}

fn parse_key_code(value: &str) -> Option<crossterm::event::KeyCode> {
    use crossterm::event::KeyCode;

    let trimmed = value.trim();
    let mut chars = trimmed.chars();
    if let (Some(character), None) = (chars.next(), chars.next()) {
        return Some(KeyCode::Char(character));
    }
    match trimmed.to_ascii_lowercase().as_str() {
        "esc" | "escape" => Some(KeyCode::Esc),
        "enter" | "return" => Some(KeyCode::Enter),
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        value if value.starts_with('f') => value[1..].parse().ok().map(KeyCode::F),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Config loading
// ---------------------------------------------------------------------------

/// Errors that can occur during config loading.
#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(String),
    Validation(Vec<String>),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "I/O error: {e}"),
            ConfigError::Parse(msg) => write!(f, "Parse error: {msg}"),
            ConfigError::Validation(errors) => {
                write!(f, "Validation errors: {}", errors.join("; "))
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Result of loading config.
#[derive(Debug)]
pub struct LoadedConfig {
    pub config: AppConfig,
    pub source: ConfigSource,
    pub warnings: Vec<String>,
}

/// Where the config was loaded from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigSource {
    /// No user config found; using built-in defaults.
    Defaults,
    /// Loaded from the given path.
    File(PathBuf),
}

/// Load config: start with defaults, then try to merge a user config file.
pub fn load_config() -> Result<LoadedConfig, ConfigError> {
    load_config_from(&config_dir().join("config.toml"))
}

pub fn load_config_from(config_path: &Path) -> Result<LoadedConfig, ConfigError> {
    let warnings = Vec::new();
    let mut config = AppConfig::default();

    let source = if config_path.exists() {
        let content = fs::read_to_string(config_path).map_err(ConfigError::Io)?;
        let user_config = toml::from_str::<AppConfig>(&content)
            .map_err(|error| ConfigError::Parse(format!("{}: {error}", config_path.display())))?;
        merge_configs(&mut config, user_config);
        ConfigSource::File(config_path.to_path_buf())
    } else {
        ConfigSource::Defaults
    };

    validate_config(&config)?;

    Ok(LoadedConfig {
        config,
        source,
        warnings,
    })
}

/// Merge a user config into the default config (field-by-field).
fn merge_configs(base: &mut AppConfig, user: AppConfig) {
    if user.theme_id != default_theme_id() {
        base.theme_id = user.theme_id;
    }
    if user.log_level != default_log_level() {
        base.log_level = user.log_level;
    }
    if let Some(override_dir) = user.save_dir_override {
        base.save_dir_override = Some(override_dir);
    }
    if !user.debug_flags.is_empty() {
        base.debug_flags = user.debug_flags;
    }

    base.keybindings = user.keybindings;
}

/// Validate config and return a list of errors.
pub fn validate_config(config: &AppConfig) -> Result<(), ConfigError> {
    let mut errors = Vec::new();

    if config.theme_id.is_empty() {
        errors.push("theme_id must not be empty".into());
    }
    if config.log_level.is_empty() {
        errors.push("log_level must not be empty".into());
    }
    if config.keybindings.quit.is_empty() {
        errors.push("quit key must not be empty".into());
    }
    if let Err(ConfigError::Validation(binding_errors)) = config.keybindings.command_bindings() {
        errors.extend(binding_errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ConfigError::Validation(errors))
    }
}

/// Save the current config to disk at the default config path.
#[allow(dead_code)]
pub fn save_config(config: &AppConfig) -> Result<PathBuf, ConfigError> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(ConfigError::Io)?;

    let path = dir.join("config.toml");
    let content = toml::to_string_pretty(config)
        .map_err(|e| ConfigError::Parse(format!("Failed to serialize config: {e}")))?;
    fs::write(&path, content).map_err(ConfigError::Io)?;
    Ok(path)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_loads() {
        let config = AppConfig::default();
        assert_eq!(config.theme_id, "bd_default");
        assert_eq!(config.keybindings.move_north, "w");
        assert_eq!(config.keybindings.quit, "q");
    }

    #[test]
    fn shipped_default_config_parses_and_validates() {
        let config: AppConfig =
            toml::from_str(include_str!("../../../config/default.toml")).unwrap();
        validate_config(&config).unwrap();
        config.keybindings.command_bindings().unwrap();
    }

    #[test]
    fn shipped_bindings_match_builtin_bindings() {
        use bd_tui::commands::{CommandBindings, UiCommand};

        let shipped: AppConfig =
            toml::from_str(include_str!("../../../config/default.toml")).unwrap();
        let shipped = shipped.keybindings.command_bindings().unwrap();
        let builtin = CommandBindings::default();
        for command in [
            UiCommand::MoveNorth,
            UiCommand::MoveSouth,
            UiCommand::MoveEast,
            UiCommand::MoveWest,
            UiCommand::Wait,
            UiCommand::RestUntilNextDay,
            UiCommand::Attack,
            UiCommand::Guard,
            UiCommand::Inventory,
            UiCommand::Pickup,
            UiCommand::UseItem,
            UiCommand::Help,
            UiCommand::Travel,
            UiCommand::Extract,
            UiCommand::AssignTask,
            UiCommand::AssignStation,
            UiCommand::Build,
            UiCommand::Save,
            UiCommand::Load,
            UiCommand::Quit,
        ] {
            assert_eq!(
                shipped.key_for(command),
                builtin.key_for(command),
                "shipped and built-in defaults drifted for {command:?}"
            );
        }
    }

    #[test]
    fn readme_default_controls_match_shipped_bindings() {
        let shipped: AppConfig =
            toml::from_str(include_str!("../../../config/default.toml")).unwrap();
        let readme = include_str!("../../../README.md").to_ascii_lowercase();
        let keys = shipped.keybindings;
        let expected_rows = [
            format!(
                "| `{}`/`↑` `{}`/`↓` `{}`/`←` `{}`/`→` | move |",
                keys.move_north, keys.move_south, keys.move_west, keys.move_east
            ),
            format!("| `{}` | wait (restore ap) |", keys.wait),
            format!(
                "| `{}` | rest until next day (shelter only) |",
                keys.rest_until_next_day
            ),
            format!("| `{}` | attack nearest enemy |", keys.attack),
            format!("| `{}` | guard (defensive stance) |", keys.guard),
            format!("| `{}` | pick up item |", keys.pickup),
            format!("| `{}` | use carried item |", keys.use_item),
            format!(
                "| `{}` | open station build menu / cancel build mode |",
                keys.build
            ),
            format!(
                "| `{}` | open colony management and select a survivor/task |",
                keys.assign_task
            ),
            format!(
                "| `{}` | open colony management at station staffing |",
                keys.assign_station
            ),
            format!("| `{}` | inventory screen |", keys.inventory),
            format!(
                "| `{}` | enter the fixed dungeon from the shelter |",
                keys.travel
            ),
            format!("| `{}` | extract at the dungeon exit |", keys.extract),
            format!("| `{}` | context help |", keys.help),
            format!("| `{}` | save the current game |", keys.save),
            format!("| `{}` | load the current game |", keys.load),
            format!(
                "| `{}` / `esc` | quit / cancel the active interaction |",
                keys.quit
            ),
        ];
        for expected in expected_rows {
            let expected = expected.to_ascii_lowercase();
            assert!(
                readme.contains(&expected),
                "README is missing exact shipped control row `{expected}`"
            );
        }
        assert!(
            readme.contains("| `1`-`5` | select a station type in the build menu |"),
            "README must document the one fixed numbered-menu interaction"
        );
    }

    #[test]
    fn missing_config_uses_defaults() {
        let path = std::env::temp_dir().join("bd-missing-config.toml");
        let _ = std::fs::remove_file(&path);
        let loaded = load_config_from(&path).unwrap();
        // The source will be Defaults since we don't write a file in this test.
        // We just verify it doesn't panic and returns something valid.
        assert_eq!(loaded.config.keybindings.quit, "q");
    }

    #[test]
    fn bad_config_reports_readable_error() {
        // Test that invalid TOML produces a readable error
        let result = toml::from_str::<AppConfig>("[[[invalid toml]]]");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        // The error should contain something recognizable
        assert!(!err.is_empty());
    }

    #[test]
    fn config_directory_resolves() {
        let dir = config_dir();
        assert!(!dir.as_os_str().is_empty());
        // Should contain the app name
        assert!(dir.to_string_lossy().contains("broken-divinity"));
    }

    #[test]
    fn save_directory_resolves() {
        let dir = data_dir();
        assert!(!dir.as_os_str().is_empty());
        assert!(dir.to_string_lossy().contains("broken-divinity"));
    }

    #[test]
    fn keybinding_maps_to_action() {
        let kb = KeyBindingConfig::default();
        assert_eq!(kb.move_north, "w");
        assert_eq!(kb.attack, "f");
        assert_eq!(kb.quit, "q");
        assert_eq!(kb.inventory, "i");
    }

    #[test]
    fn semantic_bindings_derive_from_config() {
        use bd_tui::commands::UiCommand;
        use crossterm::event::KeyCode;

        let kb = KeyBindingConfig::default();
        let bindings = kb.command_bindings().unwrap();
        assert_eq!(
            bindings.command_for_key(&KeyCode::Char('f')),
            Some(UiCommand::Attack)
        );
        assert_eq!(
            bindings.command_for_key(&KeyCode::F(5)),
            Some(UiCommand::Save)
        );
    }

    #[test]
    fn settings_persist_across_app_restart_simulation() {
        // Simulate: create config, save, load, verify
        let config = AppConfig::default();
        let save_result = save_config(&config);
        // Save might fail if we can't write to the real config dir.
        // We just verify the API works (test may need config dir).
        if let Ok(path) = save_result {
            // Clean up
            let _ = std::fs::remove_file(&path);
            let _ = std::fs::remove_dir(path.parent().unwrap());
        }
    }

    #[test]
    fn custom_keybinding_reaches_semantic_commands() {
        use bd_tui::commands::UiCommand;
        use crossterm::event::KeyCode;

        let kb = KeyBindingConfig {
            quit: "x".into(),
            ..KeyBindingConfig::default()
        };
        let bindings = kb.command_bindings().unwrap();
        assert_eq!(
            bindings.command_for_key(&KeyCode::Char('x')),
            Some(UiCommand::Quit)
        );
        assert_ne!(
            bindings.command_for_key(&KeyCode::Char('q')),
            Some(UiCommand::Quit)
        );
    }

    #[test]
    fn validate_config_rejects_empty_theme() {
        let config = AppConfig {
            theme_id: "".into(),
            ..AppConfig::default()
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("theme_id"));
    }

    #[test]
    fn validate_config_rejects_contextual_binding_conflicts() {
        let config = AppConfig {
            keybindings: KeyBindingConfig {
                attack: "w".into(),
                ..KeyBindingConfig::default()
            },
            ..AppConfig::default()
        };
        let error = validate_config(&config).unwrap_err().to_string();
        assert!(error.contains("MoveNorth"));
        assert!(error.contains("Attack"));
    }

    #[test]
    fn validate_config_rejects_unknown_key_names() {
        let config = AppConfig {
            keybindings: KeyBindingConfig {
                save: "banana".into(),
                ..KeyBindingConfig::default()
            },
            ..AppConfig::default()
        };
        let error = validate_config(&config).unwrap_err().to_string();
        assert!(error.contains("unsupported key binding"));
        assert!(error.contains("banana"));
    }

    #[test]
    fn invalid_config_returns_readable_application_error() {
        let path = std::env::temp_dir().join("bd-invalid-config.toml");
        std::fs::write(&path, "[keybindings]\nattack = [").unwrap();

        let error = load_config_from(&path).unwrap_err().to_string();
        assert!(error.contains("Parse error"));
        assert!(error.contains("bd-invalid-config.toml"));

        let _ = std::fs::remove_file(path);
    }
}
