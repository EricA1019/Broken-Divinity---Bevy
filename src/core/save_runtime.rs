use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core_state::AppState;

const SAVE_FILE_NAME: &str = "./broken_divinity_save.json";

#[derive(Resource, Debug, Clone, Default)]
pub struct PendingLoad(pub Option<SaveGame>);

impl PendingLoad {
    pub fn take(&mut self) -> Option<SaveGame> {
        self.0.take()
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct PlayerSnapshot(pub Option<SavePlayerState>);

#[derive(Resource, Debug, Clone, Default)]
pub struct SaveAndQuitRequested(pub bool);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadGameError {
    MissingSave,
    InvalidData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SaveAppState {
    #[default]
    Menu,
    Overworld,
    Dungeon,
    Colony,
    Combat,
    GameOver,
}

impl SaveAppState {
    pub fn into_runtime_state(self) -> AppState {
        match self {
            Self::Menu | Self::Colony | Self::GameOver => AppState::Colony,
            Self::Overworld => AppState::Overworld,
            Self::Dungeon | Self::Combat => AppState::Dungeon,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SaveGameTime {
    #[serde(default)]
    pub turn: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SavePlayerState {
    #[serde(default)]
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SaveGame {
    #[serde(default)]
    pub seed: u64,
    #[serde(default)]
    pub app_state: SaveAppState,
    #[serde(default)]
    pub game_time: SaveGameTime,
    #[serde(default)]
    pub player: SavePlayerState,
}

fn save_path() -> PathBuf {
    PathBuf::from(SAVE_FILE_NAME)
}

pub fn save_exists() -> bool {
    save_path().exists()
}

pub fn load_game() -> Option<SaveGame> {
    load_game_detailed().ok()
}

pub fn load_game_detailed() -> Result<SaveGame, LoadGameError> {
    let raw = std::fs::read_to_string(save_path()).map_err(|_| LoadGameError::MissingSave)?;
    serde_json::from_str(&raw).map_err(|_| LoadGameError::InvalidData)
}

pub fn queue_loaded_game(commands: &mut Commands, save: SaveGame) {
    commands.insert_resource(PendingLoad(Some(save)));
}

pub fn load_success_message() -> String {
    "Load complete. Recap restored for the active run.".to_string()
}

pub fn load_error_message(error: LoadGameError) -> String {
    match error {
        LoadGameError::MissingSave => {
            "Load failed: no save file was found for this run.".to_string()
        }
        LoadGameError::InvalidData => {
            "Load failed: the save data could not be read safely.".to_string()
        }
    }
}