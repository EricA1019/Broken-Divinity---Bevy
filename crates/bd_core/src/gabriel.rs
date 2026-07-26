//! Gabriel encounter — first dungeon meeting with the mysterious entity.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

pub const GABRIEL_TRIGGER_FLOOR: u32 = 1;
pub const GABRIEL_SANITY_RECOVERY: i32 = 10;

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize)]
pub struct GabrielState {
    pub appeared: bool,
    pub accepted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gabriel_starts_hidden() {
        let state = GabrielState::default();
        assert!(!state.appeared);
        assert!(!state.accepted);
    }
}
