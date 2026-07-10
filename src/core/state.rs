use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum AppState {
    #[default]
    Menu,
    Overworld,
    Dungeon,
    Colony,
    /// Active turn-based combat on the shelter map during raids.
    /// Not yet wired — reserved for Phase 3 shelter defense.
    /// On load, maps back to Dungeon/Colony as raids aren't saveable mid-combat.
    Combat,
    GameOver,
}
