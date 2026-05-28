use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
pub enum AppState {
    #[default]
    Menu,
    Colony,
    Overworld,
    Dungeon,
    Combat,
    GameOver,
}
