//! Shelter economy — tracked resources and production/consumption.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

const COLONY_TICK_SECONDS: f32 = 0.5;
const TRAVEL_DAY_SECONDS: f32 = 0.5;

/// World seed used for deterministic procgen across a run.
#[derive(Resource, Debug, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct WorldSeed(pub u64);

/// Placeholder atlas handle for tilemaps until authored art is wired back in.
#[derive(Resource, Debug, Clone, Default)]
pub struct PlaceholderTileAtlas(pub Handle<Image>);

#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct ColonyTickTimer(pub Timer);

impl Default for ColonyTickTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            COLONY_TICK_SECONDS,
            TimerMode::Repeating,
        ))
    }
}

#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct TravelDayTimer(pub Timer);

impl Default for TravelDayTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            TRAVEL_DAY_SECONDS,
            TimerMode::Repeating,
        ))
    }
}

pub fn reset_colony_tick_timer(mut timer: ResMut<ColonyTickTimer>) {
    timer.0.reset();
}

pub fn tick_colony_timer(time: Res<Time>, mut timer: ResMut<ColonyTickTimer>) {
    timer.0.tick(time.delta());
}

pub fn colony_tick_ready(timer: Res<ColonyTickTimer>) -> bool {
    timer.0.just_finished()
}

pub fn reset_travel_day_timer(mut timer: ResMut<TravelDayTimer>) {
    timer.0.reset();
}

pub fn tick_travel_day_timer(time: Res<Time>, mut timer: ResMut<TravelDayTimer>) {
    timer.0.tick(time.delta());
}

pub fn travel_day_ready(timer: Res<TravelDayTimer>) -> bool {
    timer.0.just_finished()
}

/// Global shelter resource stockpile.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct ShelterResources {
    pub food: u32,
    pub water: u32,
    pub scrap: u32,
    pub medicine: u32,
    pub ammo: u32,
}

impl ShelterResources {
    /// Starting resources for a new game.
    pub fn new_game() -> Self {
        Self {
            food: 10,
            water: 10,
            scrap: 15,
            medicine: 3,
            ammo: 10,
        }
    }

    /// Attempt to consume `amount` of `resource`. Returns `true` if successful.
    pub fn try_consume(&mut self, resource: ResourceKind, amount: u32) -> bool {
        let stock = self.get_mut(resource);
        if *stock >= amount {
            *stock -= amount;
            true
        } else {
            false
        }
    }

    /// Add `amount` to the given resource stockpile.
    pub fn add(&mut self, resource: ResourceKind, amount: u32) {
        *self.get_mut(resource) += amount;
    }

    pub fn get(&self, resource: ResourceKind) -> u32 {
        match resource {
            ResourceKind::Food => self.food,
            ResourceKind::Water => self.water,
            ResourceKind::Scrap => self.scrap,
            ResourceKind::Medicine => self.medicine,
            ResourceKind::Ammo => self.ammo,
        }
    }

    fn get_mut(&mut self, resource: ResourceKind) -> &mut u32 {
        match resource {
            ResourceKind::Food => &mut self.food,
            ResourceKind::Water => &mut self.water,
            ResourceKind::Scrap => &mut self.scrap,
            ResourceKind::Medicine => &mut self.medicine,
            ResourceKind::Ammo => &mut self.ammo,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum ResourceKind {
    Food,
    Water,
    Scrap,
    Medicine,
    Ammo,
}

impl ResourceKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Food => "Food",
            Self::Water => "Water",
            Self::Scrap => "Scrap",
            Self::Medicine => "Medicine",
            Self::Ammo => "Ammo",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_resources() {
        let r = ShelterResources::new_game();
        assert_eq!(r.food, 10);
        assert_eq!(r.water, 10);
        assert_eq!(r.scrap, 15);
        assert_eq!(r.medicine, 3);
        assert_eq!(r.ammo, 10);
    }

    #[test]
    fn test_try_consume_success() {
        let mut r = ShelterResources::new_game();
        assert!(r.try_consume(ResourceKind::Food, 5));
        assert_eq!(r.food, 5);
    }

    #[test]
    fn test_try_consume_fail() {
        let mut r = ShelterResources::new_game();
        assert!(!r.try_consume(ResourceKind::Medicine, 10));
        assert_eq!(r.medicine, 3); // unchanged
    }

    #[test]
    fn test_add() {
        let mut r = ShelterResources::new_game();
        r.add(ResourceKind::Scrap, 5);
        assert_eq!(r.scrap, 20);
    }
}
