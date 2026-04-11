use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use broken_divinity::core::components::{Player, Position, TileKind};
use broken_divinity::core::inventory::{Equipment, Inventory, RangedWeaponState};
use broken_divinity::core::movement::MapTiles;
use broken_divinity::core::perks::PlayerPerks;
use broken_divinity::core::sanity::RaidExposure;
use broken_divinity::core::state::AppState;
use broken_divinity::core::stats::{CombatStats, EntityName};
use broken_divinity::core::turn::TurnPhase;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(broken_divinity::core::plugin)
        .add_plugins(broken_divinity::game::colony::plugin)
        .add_plugins(broken_divinity::game::combat::plugin)
        .add_plugins(broken_divinity::game::dungeon::plugin)
        .add_plugins(broken_divinity::game::overworld::plugin)
        .init_state::<AppState>()
        // --- State wrapper registrations for BRP visibility ---
        .register_type::<State<AppState>>()
        .register_type::<NextState<AppState>>()
        .register_type::<State<TurnPhase>>()
        .register_type::<NextState<TurnPhase>>()
        // --- Action resources ---
        .init_resource::<broken_divinity::ui::gabriel_dialogue_panel::GabrielDialogueUiAction>()
        .init_resource::<broken_divinity::ui::inventory_panel::InventoryOpen>()
        .init_resource::<broken_divinity::ui::inventory_panel::InventoryUiAction>()
        .init_resource::<broken_divinity::ui::journal_panel::JournalOpen>()
        .init_resource::<broken_divinity::ui::menu::MenuUiAction>()
        .init_resource::<broken_divinity::ui::overworld_panel::OverworldUiAction>()
        .init_resource::<broken_divinity::ui::colony_panel::ColonyUiAction>()
        .init_resource::<broken_divinity::ui::gameover::GameOverUiAction>()
        .init_resource::<broken_divinity::ui::perk_choice_panel::PerkChoiceUiAction>()
        // --- State transitions ---
        .add_systems(OnEnter(AppState::Menu), broken_divinity::core::save::reset_run_state_for_menu)
        .add_systems(OnEnter(AppState::Colony), broken_divinity::core::save::autosave)
        // --- Draw systems — EguiPrimaryContextPass (read-only) ---
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::menu::draw_main_menu.run_if(in_state(AppState::Menu)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::overworld_panel::draw_overworld_panel
                .run_if(in_state(AppState::Overworld)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::gabriel_dialogue_panel::draw_gabriel_dialogue_panel
                .run_if(in_state(AppState::Dungeon)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::gamelog_panel::draw_gamelog_panel,
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::gameover::draw_gameover_screen,
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::hud::draw_hud
                .run_if(in_state(AppState::Dungeon).or(in_state(AppState::Colony))),
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::perk_choice_panel::draw_perk_choice_panel
                .run_if(in_state(AppState::Dungeon).or(in_state(AppState::Colony))),
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::inventory_panel::draw_inventory_panel
                .run_if(in_state(AppState::Dungeon).or(in_state(AppState::Colony))),
        )
        .add_systems(
            EguiPrimaryContextPass,
            broken_divinity::ui::journal_panel::draw_journal_panel
                .run_if(in_state(AppState::Dungeon)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                broken_divinity::ui::colony_panel::draw_resource_bar,
                broken_divinity::ui::colony_panel::draw_survivor_panel,
                broken_divinity::ui::colony_panel::draw_build_panel,
            )
                .run_if(in_state(AppState::Colony)),
        )
        // --- Process systems — Update (mutations) ---
        .add_systems(
            Update,
            broken_divinity::ui::menu::process_menu_action.run_if(in_state(AppState::Menu)),
        )
        .add_systems(
            Update,
            broken_divinity::ui::overworld_panel::process_overworld_action
                .run_if(in_state(AppState::Overworld)),
        )
        .add_systems(
            Update,
            broken_divinity::ui::gabriel_dialogue_panel::process_gabriel_dialogue_action
                .run_if(in_state(AppState::Dungeon)),
        )
        .add_systems(
            Update,
            broken_divinity::ui::gameover::process_gameover_action,
        )
        .add_systems(
            Update,
            broken_divinity::ui::gameover::check_player_death
                .run_if(in_state(AppState::Dungeon)),
        )
        .add_systems(
            Update,
            broken_divinity::ui::perk_choice_panel::process_perk_choice_action
                .run_if(in_state(AppState::Dungeon).or(in_state(AppState::Colony))),
        )
        .add_systems(
            Update,
            broken_divinity::ui::inventory_panel::process_inventory_action
                .run_if(in_state(AppState::Dungeon).or(in_state(AppState::Colony))),
        )
        .add_systems(
            Update,
            broken_divinity::ui::colony_panel::process_colony_action
                .run_if(in_state(AppState::Colony)),
        )
        // --- Input handlers — Update ---
        .add_systems(
            Update,
            broken_divinity::ui::inventory_panel::toggle_inventory
                .run_if(in_state(AppState::Dungeon).or(in_state(AppState::Colony))),
        )
        .add_systems(
            Update,
            broken_divinity::ui::journal_panel::toggle_journal
                .run_if(in_state(AppState::Dungeon)),
        )
        .add_systems(
            Update,
            enter_overworld_from_colony.run_if(in_state(AppState::Colony)),
        )
        .add_systems(
            Update,
            broken_divinity::core::save::handle_save_and_quit
                .run_if(in_state(AppState::Colony).or(in_state(AppState::Overworld))),
        );

    // BRP — Bevy Remote Protocol for live state inspection (dev builds only)
    #[cfg(feature = "dev")]
    {
        app.add_plugins(bevy::remote::RemotePlugin::default());
        app.add_plugins(bevy::remote::http::RemoteHttpPlugin::default());
    }

    app.run();
}

/// Transition from Colony → Overworld when the player is on the gate tile and presses Enter.
fn enter_overworld_from_colony(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    player_q: Query<
        (
            &Position,
            &CombatStats,
            &Inventory,
            &Equipment,
            &RangedWeaponState,
            &RaidExposure,
            &PlayerPerks,
            Option<&EntityName>,
            &broken_divinity::core::abilities::SprintCooldown,
        ),
        With<Player>,
    >,
    map: Option<Res<MapTiles>>,
) {
    if !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }
    let Ok((pos, stats, inventory, equipment, ranged_state, sanity, perks, name, sprint_cd)) = player_q.single() else {
        return;
    };
    let Some(map) = map else { return };
    if let Some(TileKind::StairsUp) = map.get_tile(pos.x, pos.y) {
        commands.insert_resource(broken_divinity::core::save::PlayerSnapshot(Some(
            broken_divinity::core::save::snapshot_player_state(
                pos,
                stats,
                inventory,
                equipment,
                ranged_state,
                sanity,
                perks,
                name,
                sprint_cd.remaining,
            ),
        )));
        next_state.set(AppState::Overworld);
    }
}
