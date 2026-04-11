//! Grid-based movement system.
//!
//! Reads WASD / arrow key input, moves `Position` by 1 tile if the target is walkable.
//! If the target tile has an enemy, triggers a bump-attack instead of moving.
//! Runs only in `AppState::Dungeon`.

use bevy::prelude::*;
use crate::core::abilities::SprintCooldown;
use crate::core::components::{Enemy, Player, Position, TileKind};
use crate::core::gamelog::{GameLog, LogColor};
use crate::core::perks::PendingPerkChoices;
use crate::core::sanity::{forced_move_direction, should_lose_control, RaidExposure, SanityThreshold};
use crate::core::state::AppState;
use crate::core::turn::{GameTime, TurnPhase};
use crate::game::dungeon::gabriel::GabrielDialogueState;

/// Resource holding the current dungeon floor tiles for collision checks.
#[derive(Resource, Debug, Clone)]
pub struct MapTiles {
    pub tiles: Vec<Vec<TileKind>>,
    pub width: usize,
    pub height: usize,
}

impl MapTiles {
    pub fn new(tiles: Vec<Vec<TileKind>>) -> Self {
        let height = tiles.len();
        let width = tiles.first().map(|r| r.len()).unwrap_or(0);
        Self { tiles, width, height }
    }

    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return false;
        }
        matches!(
            self.tiles[y as usize][x as usize],
            TileKind::Floor | TileKind::Door | TileKind::StairsUp | TileKind::StairsDown
        )
    }

    pub fn get_tile(&self, x: i32, y: i32) -> Option<TileKind> {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return None;
        }
        Some(self.tiles[y as usize][x as usize])
    }
}

/// System: read keyboard input and move the player on the grid.
/// If bump target is an enemy, set BumpAttackTarget instead of moving.
/// Shift + direction triggers a 2-tile sprint (3-turn cooldown).
pub fn grid_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    map: Option<Res<MapTiles>>,
    pending_perks: Option<Res<PendingPerkChoices>>,
    dialogue_state: Option<Res<GabrielDialogueState>>,
    game_time: Res<GameTime>,
    app_state: Res<State<AppState>>,
    turn_phase: Res<State<TurnPhase>>,
    mut next_turn_phase: ResMut<NextState<TurnPhase>>,
    mut query: Query<(&mut Position, Option<&RaidExposure>, &mut SprintCooldown), With<Player>>,
    enemies: Query<&Position, (With<Enemy>, Without<Player>, Without<crate::game::dungeon::gabriel::Gabriel>)>,
    mut bump_target: Option<ResMut<crate::game::dungeon::melee::BumpAttackTarget>>,
    mut log: Option<ResMut<GameLog>>,
) {
    let Ok((mut pos, exposure, mut sprint_cd)) = query.single_mut() else { return; };
    let Some(map) = map else { return; };

    if pending_perks.is_some_and(|pending| pending.has_pending()) {
        return;
    }

    if dialogue_state.is_some_and(|dialogue| dialogue.is_active()) {
        return;
    }

    let (mut dx, mut dy) = (0, 0);
    if keyboard.just_pressed(KeyCode::KeyW) || keyboard.just_pressed(KeyCode::ArrowUp) {
        dy = -1;
    }
    if keyboard.just_pressed(KeyCode::KeyS) || keyboard.just_pressed(KeyCode::ArrowDown) {
        dy = 1;
    }
    if keyboard.just_pressed(KeyCode::KeyA) || keyboard.just_pressed(KeyCode::ArrowLeft) {
        dx = -1;
    }
    if keyboard.just_pressed(KeyCode::KeyD) || keyboard.just_pressed(KeyCode::ArrowRight) {
        dx = 1;
    }

    if dx == 0 && dy == 0 {
        return;
    }

    if exposure.is_some_and(|exp| exp.threshold() == SanityThreshold::Breaking)
        && should_lose_control(game_time.turn)
    {
        (dx, dy) = forced_move_direction(game_time.turn);
    }

    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let wants_sprint = shift_held && sprint_cd.remaining == 0;

    let nx = pos.x + dx;
    let ny = pos.y + dy;

    // Check if an enemy occupies the target tile
    let has_enemy = enemies.iter().any(|ep| ep.x == nx && ep.y == ny);
    if has_enemy {
        if let Some(ref mut bt) = bump_target {
            bt.0 = Some(Position::new(nx, ny));
        }
        if *app_state.get() == AppState::Dungeon
            && *turn_phase.get() == TurnPhase::AwaitingInput
        {
            next_turn_phase.set(TurnPhase::PlayerTurn);
        }
        return;
    }

    if map.is_walkable(nx, ny) {
        if wants_sprint {
            let nx2 = pos.x + dx * 2;
            let ny2 = pos.y + dy * 2;
            let tile2_enemy = enemies.iter().any(|ep| ep.x == nx2 && ep.y == ny2);
            if map.is_walkable(nx2, ny2) && !tile2_enemy {
                // Sprint: move 2 tiles
                pos.x = nx2;
                pos.y = ny2;
                sprint_cd.remaining = 3;
                if let Some(ref mut log) = log {
                    log.push("You sprint forward!", LogColor::System, game_time.turn);
                }
            } else {
                // Second tile blocked — fall back to normal 1-tile move
                pos.x = nx;
                pos.y = ny;
            }
        } else {
            pos.x = nx;
            pos.y = ny;
        }

        if *app_state.get() == AppState::Dungeon
            && *turn_phase.get() == TurnPhase::AwaitingInput
        {
            next_turn_phase.set(TurnPhase::PlayerTurn);
        }
    }
}

/// Syncs the Bevy `Transform` from the grid `Position`.
pub fn sync_position_to_transform(
    mut query: Query<(&Position, &mut Transform), Changed<Position>>,
) {
    for (pos, mut tf) in query.iter_mut() {
        tf.translation.x = pos.x as f32 * 16.0;
        tf.translation.y = pos.y as f32 * 16.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::abilities::SprintCooldown;
    use crate::core::components::{Player, Position, TileKind};
    use crate::core::turn::TurnPhase;

    fn floor_map(w: usize, h: usize) -> MapTiles {
        MapTiles::new(vec![vec![TileKind::Floor; w]; h])
    }

    /// Helper: directly apply sprint/move logic on the world.
    /// Returns the player's position after the attempt.
    fn sprint_attempt(app: &mut App, dx: i32, dy: i32, shift: bool) -> Position {
        let map = app.world().resource::<MapTiles>().clone();

        app.world_mut().resource_scope(|world, mut log: Mut<GameLog>| {
            let mut query = world.query_filtered::<(&mut Position, &mut SprintCooldown), With<Player>>();
            for (mut pos, mut sprint_cd) in query.iter_mut(world) {
                let wants_sprint = shift && sprint_cd.remaining == 0;
                let nx = pos.x + dx;
                let ny = pos.y + dy;

                if map.is_walkable(nx, ny) {
                    if wants_sprint {
                        let nx2 = pos.x + dx * 2;
                        let ny2 = pos.y + dy * 2;
                        if map.is_walkable(nx2, ny2) {
                            pos.x = nx2;
                            pos.y = ny2;
                            sprint_cd.remaining = 3;
                            log.push("You sprint forward!", LogColor::System, 0);
                        } else {
                            pos.x = nx;
                            pos.y = ny;
                        }
                    } else {
                        pos.x = nx;
                        pos.y = ny;
                    }
                }
            }
        });

        let mut query = app.world_mut().query_filtered::<&Position, With<Player>>();
        query.single(app.world()).unwrap().clone()
    }

    #[test]
    fn sprint_moves_two_tiles_when_cooldown_zero() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let map = floor_map(10, 10);
        app.insert_resource(map);
        app.insert_resource(GameLog::default());

        app.world_mut().spawn((
            Player,
            Position::new(5, 5),
            SprintCooldown { remaining: 0 },
        ));

        let result_pos = sprint_attempt(&mut app, 1, 0, true);
        assert_eq!(result_pos, Position::new(7, 5), "Sprint should move 2 tiles");
    }

    #[test]
    fn sprint_sets_cooldown_to_three() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let map = floor_map(10, 10);
        app.insert_resource(map);
        app.insert_resource(GameLog::default());

        let entity = app.world_mut().spawn((
            Player,
            Position::new(5, 5),
            SprintCooldown { remaining: 0 },
        )).id();

        sprint_attempt(&mut app, 1, 0, true);

        let cd = app.world().get::<SprintCooldown>(entity).unwrap();
        assert_eq!(cd.remaining, 3, "Sprint should set cooldown to 3");
    }

    #[test]
    fn sprint_blocked_when_on_cooldown() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let map = floor_map(10, 10);
        app.insert_resource(map);
        app.insert_resource(GameLog::default());

        app.world_mut().spawn((
            Player,
            Position::new(5, 5),
            SprintCooldown { remaining: 2 },
        ));

        let result_pos = sprint_attempt(&mut app, 1, 0, true);
        assert_eq!(result_pos, Position::new(6, 5), "Should move only 1 tile on cooldown");
    }

    #[test]
    fn cooldown_ticks_down_each_turn() {
        use bevy::ecs::system::RunSystemOnce;
        use crate::core::turn::tick_sprint_cooldown;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = app.world_mut().spawn(SprintCooldown { remaining: 3 }).id();

        app.world_mut().run_system_once(tick_sprint_cooldown);
        assert_eq!(app.world().get::<SprintCooldown>(entity).unwrap().remaining, 2);

        app.world_mut().run_system_once(tick_sprint_cooldown);
        assert_eq!(app.world().get::<SprintCooldown>(entity).unwrap().remaining, 1);

        app.world_mut().run_system_once(tick_sprint_cooldown);
        assert_eq!(app.world().get::<SprintCooldown>(entity).unwrap().remaining, 0);

        app.world_mut().run_system_once(tick_sprint_cooldown);
        assert_eq!(app.world().get::<SprintCooldown>(entity).unwrap().remaining, 0);
    }

    #[test]
    fn sprint_fallback_when_second_tile_blocked() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut map = floor_map(10, 10);
        map.tiles[5][7] = TileKind::Wall;
        app.insert_resource(map);
        app.insert_resource(GameLog::default());

        let entity = app.world_mut().spawn((
            Player,
            Position::new(5, 5),
            SprintCooldown { remaining: 0 },
        )).id();

        let result_pos = sprint_attempt(&mut app, 1, 0, true);
        assert_eq!(result_pos, Position::new(6, 5), "Should fall back to 1-tile move");

        let cd = app.world().get::<SprintCooldown>(entity).unwrap();
        assert_eq!(cd.remaining, 0, "Cooldown should not trigger on fallback");
    }
}
