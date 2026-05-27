#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveRecapState {
    Colony,
    Overworld,
    Dungeon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecapRisk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveRecap {
    pub state: SaveRecapState,
    pub risk: RecapRisk,
    pub next_step: &'static str,
}

const COLONY_NEXT_STEP: &str = "Assign station tasks and prepare supplies.";
const OVERWORLD_NEXT_STEP: &str = "Select destination and confirm travel route.";
const DUNGEON_NEXT_STEP: &str = "Regroup, scout nearby tiles, then advance carefully.";

pub fn recap_for_state(state: SaveRecapState) -> SaveRecap {
    match state {
        SaveRecapState::Colony => SaveRecap {
            state,
            risk: RecapRisk::Low,
            next_step: COLONY_NEXT_STEP,
        },
        SaveRecapState::Overworld => SaveRecap {
            state,
            risk: RecapRisk::Medium,
            next_step: OVERWORLD_NEXT_STEP,
        },
        SaveRecapState::Dungeon => SaveRecap {
            state,
            risk: RecapRisk::High,
            next_step: DUNGEON_NEXT_STEP,
        },
    }
}

pub fn legacy_recap(legacy_state: &str) -> SaveRecap {
    let normalized = legacy_state.to_ascii_lowercase();
    if normalized == "overworld" {
        return recap_for_state(SaveRecapState::Overworld);
    }

    if normalized == "dungeon" {
        return recap_for_state(SaveRecapState::Dungeon);
    }

    recap_for_state(SaveRecapState::Colony)
}
