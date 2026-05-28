#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use pathfinding::prelude::astar;
use serde::{Deserialize, Serialize};

use crate::core::components::{Enemy, Player, Position, TileKind};
use crate::core::gamelog::{GameLog, LogColor};
use crate::core::movement::MapTiles;
use crate::core::stats::{CombatStats, EntityName};
use crate::core::turn::GameTime;
use crate::game::combat::{DamageType, calc_damage, roll_check};
use crate::game::dungeon::bsp::{DungeonFloor, Rect};
use crate::game::overworld::graphgen::DungeonStoryTag;

const GABRIEL_NAME: &str = "Gabriel";
const GABRIEL_SKILL: u32 = 55;
const GABRIEL_DAMAGE: i32 = 7;

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Default, Reflect)]
#[reflect(Resource)]
pub struct GabrielState {
    #[serde(default)]
    pub encounter_completed: bool,
    #[serde(default)]
    pub joined: bool,
}

impl GabrielState {
    pub fn should_stage_intro(
        &self,
        floor_number: u32,
        story_tag: Option<DungeonStoryTag>,
    ) -> bool {
        floor_number == 2
            && story_tag == Some(DungeonStoryTag::GabrielIntro)
            && !self.encounter_completed
            && !self.joined
    }
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Gabriel;

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct GabrielCompanion {
    pub active: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct GabrielEncounter {
    pub room: Rect,
    pub triggered: bool,
}

impl GabrielEncounter {
    pub fn new(room: Rect) -> Self {
        Self {
            room,
            triggered: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum GabrielDialogueStep {
    Warning,
    Identity,
    Threat,
    Aid,
}

#[derive(Resource, Debug, Clone, Default, Reflect)]
#[reflect(Resource)]
pub struct GabrielDialogueState(pub Option<GabrielDialogueStep>);

impl GabrielDialogueState {
    pub fn is_active(&self) -> bool {
        self.0.is_some()
    }

    pub fn open_warning(&mut self) {
        self.0 = Some(GabrielDialogueStep::Warning);
    }

    pub fn close(&mut self) {
        self.0 = None;
    }
}

pub fn select_intro_room(floor: &DungeonFloor) -> Option<Rect> {
    let stairs_room = floor.rooms.last().copied();
    floor
        .rooms
        .iter()
        .copied()
        .skip(1)
        .filter(|room| Some(*room) != stairs_room)
        .max_by_key(|room| room.w * room.h)
        .or(stairs_room)
}

pub fn companion_spawn_on_floor(floor: &DungeonFloor, anchor: (i32, i32)) -> (i32, i32) {
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (0, 0)] {
        let x = anchor.0 + dx;
        let y = anchor.1 + dy;
        if is_walkable_tile(&floor.tiles, x, y) {
            return (x, y);
        }
    }

    anchor
}

pub fn spawn_gabriel_entity(commands: &mut Commands, pos: (i32, i32), active: bool) -> Entity {
    commands
        .spawn((
            Gabriel,
            GabrielCompanion { active },
            EntityName {
                name: GABRIEL_NAME.to_string(),
            },
            Position::new(pos.0, pos.1),
            Sprite {
                color: Color::srgba(0.85, 0.85, 1.0, if active { 0.85 } else { 0.65 }),
                custom_size: Some(Vec2::new(14.0, 14.0)),
                ..Default::default()
            },
        ))
        .id()
}

pub fn start_gabriel_encounter(
    player_q: Query<&Position, With<Player>>,
    encounter: Option<ResMut<GabrielEncounter>>,
    mut dialogue: ResMut<GabrielDialogueState>,
    gabriel_state: Res<GabrielState>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    let Some(mut encounter) = encounter else {
        return;
    };
    if encounter.triggered || dialogue.is_active() || gabriel_state.joined {
        return;
    }

    let Ok(player_pos) = player_q.single() else {
        return;
    };
    if !room_contains(encounter.room, *player_pos) {
        return;
    }

    encounter.triggered = true;
    dialogue.open_warning();
    log.push(
        "A pale figure steps from the static and bars the room.",
        LogColor::Status,
        time.turn,
    );
}

pub fn gabriel_turn(
    mut commands: Commands,
    mut gabriel_q: Query<
        (&mut Position, &GabrielCompanion),
        (With<Gabriel>, Without<Enemy>, Without<Player>),
    >,
    mut enemies: ParamSet<(
        Query<(Entity, &Position, &mut CombatStats, &EntityName), (With<Enemy>, Without<Player>)>,
        Query<(Entity, &Position), (With<Enemy>, Without<Player>)>,
    )>,
    player_q: Query<&Position, (With<Player>, Without<Gabriel>)>,
    map: Option<Res<MapTiles>>,
    mut log: ResMut<GameLog>,
    time: Res<GameTime>,
) {
    let Ok((mut gabriel_pos, companion)) = gabriel_q.single_mut() else {
        return;
    };
    if !companion.active {
        return;
    }

    let Ok(player_pos) = player_q.single() else {
        return;
    };

    let mut nearest_enemy: Option<(Entity, Position, i32)> = None;
    let occupied: Vec<(i32, i32)> = enemies
        .p1()
        .iter()
        .map(|(entity, pos)| {
            let dist = (gabriel_pos.x - pos.x).abs() + (gabriel_pos.y - pos.y).abs();
            if nearest_enemy.is_none_or(|(_, _, best)| dist < best) {
                nearest_enemy = Some((entity, *pos, dist));
            }
            (pos.x, pos.y)
        })
        .collect();

    let Some((target_entity, target_pos, distance)) = nearest_enemy else {
        if let Some(map) = map.as_ref()
            && let Some(next_step) = next_step_toward(
                (gabriel_pos.x, gabriel_pos.y),
                (player_pos.x, player_pos.y),
                map,
                &occupied,
                (player_pos.x, player_pos.y),
            )
        {
            gabriel_pos.x = next_step.0;
            gabriel_pos.y = next_step.1;
        }
        return;
    };

    if distance <= 1 {
        let mut hostile_enemies = enemies.p0();
        let Ok((target_entity, _enemy_pos, mut target_stats, target_name)) =
            hostile_enemies.get_mut(target_entity)
        else {
            return;
        };

        let mut rng = rand::rng();
        let check = roll_check(GABRIEL_SKILL, 0, 0, &mut rng);
        if check.success {
            let damage = calc_damage(
                GABRIEL_DAMAGE,
                GABRIEL_SKILL,
                target_stats.ar,
                target_stats.md,
                DamageType::Thaumic,
                check.critical,
                &mut rng,
            );
            target_stats.hp -= damage;
            log.push(
                format!(
                    "Gabriel lashes {} with radiant force for {} damage.",
                    target_name.name, damage
                ),
                if check.critical {
                    LogColor::Critical
                } else {
                    LogColor::PlayerHit
                },
                time.turn,
            );
            if target_stats.is_dead() {
                log.push(
                    format!("{} falls before Gabriel's warning.", target_name.name),
                    LogColor::Death,
                    time.turn,
                );
                commands.entity(target_entity).despawn();
            }
        } else {
            log.push(
                format!("Gabriel's strike misses {}.", target_name.name),
                LogColor::Miss,
                time.turn,
            );
        }
        return;
    }

    let Some(map) = map else { return };
    if let Some(next_step) = next_step_toward(
        (gabriel_pos.x, gabriel_pos.y),
        (target_pos.x, target_pos.y),
        &map,
        &occupied,
        (target_pos.x, target_pos.y),
    ) {
        gabriel_pos.x = next_step.0;
        gabriel_pos.y = next_step.1;
    }
}

fn room_contains(room: Rect, pos: Position) -> bool {
    pos.x >= room.x && pos.x < room.x + room.w && pos.y >= room.y && pos.y < room.y + room.h
}

fn is_walkable_tile(tiles: &[Vec<TileKind>], x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || y as usize >= tiles.len() || x as usize >= tiles[0].len() {
        return false;
    }

    matches!(
        tiles[y as usize][x as usize],
        TileKind::Floor | TileKind::Door | TileKind::StairsUp | TileKind::StairsDown
    )
}

fn next_step_toward(
    start: (i32, i32),
    goal: (i32, i32),
    map: &MapTiles,
    occupied: &[(i32, i32)],
    final_goal: (i32, i32),
) -> Option<(i32, i32)> {
    let result = astar(
        &start,
        |&(x, y)| {
            let mut neighbors = Vec::new();
            for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let nx = x + dx;
                let ny = y + dy;
                if !map.is_walkable(nx, ny) {
                    continue;
                }
                let blocked = occupied
                    .iter()
                    .any(|&(ox, oy)| ox == nx && oy == ny && (nx, ny) != final_goal);
                if !blocked {
                    neighbors.push(((nx, ny), 1));
                }
            }
            neighbors
        },
        |&(x, y)| ((x - goal.0).abs() + (y - goal.1).abs()) as u32,
        |&pos| pos == goal,
    );

    result.and_then(|(path, _)| path.get(1).copied())
}
