use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
#[cfg(feature = "dev")]
use bevy::remote::RemotePlugin;
#[cfg(feature = "dev")]
use bevy::remote::http::RemoteHttpPlugin;
#[cfg(feature = "dev")]
use bevy_brp_extras::BrpExtrasPlugin;
use broken_divinity::core::escape::handle_escape_to_menu;
use broken_divinity::core::state::AppState;
use broken_divinity::core::turn::TurnPhase;
use broken_divinity::game::overworld::travel::enter_overworld_from_colony;

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
        .register_type::<State<AppState>>()
        .register_type::<NextState<AppState>>()
        .register_type::<State<TurnPhase>>()
        .register_type::<NextState<TurnPhase>>()
        .init_resource::<broken_divinity::ui::gabriel_dialogue_panel::GabrielDialogueUiAction>()
        .init_resource::<broken_divinity::ui::inventory_panel::InventoryOpen>()
        .init_resource::<broken_divinity::ui::inventory_panel::InventoryUiAction>()
        .init_resource::<broken_divinity::ui::journal_panel::JournalOpen>()
        .init_resource::<broken_divinity::ui::menu::MenuUiAction>()
        .init_resource::<broken_divinity::ui::overworld_panel::OverworldUiAction>()
        .init_resource::<broken_divinity::ui::colony_panel::ColonyUiAction>()
        .init_resource::<broken_divinity::ui::gameover::GameOverUiAction>()
        .init_resource::<broken_divinity::ui::perk_choice_panel::PerkChoiceUiAction>()
        .init_resource::<broken_divinity::ui::stats_progression_panel::StatsProgressionOpen>()
        .add_systems(
            OnEnter(AppState::Menu),
            broken_divinity::core::save::reset_run_state_for_menu,
        )
        .add_systems(
            OnEnter(AppState::Colony),
            broken_divinity::core::save::autosave,
        )
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
            broken_divinity::ui::stats_progression_panel::draw_stats_progression_panel
                .run_if(
                    in_state(AppState::Dungeon)
                        .or(in_state(AppState::Colony))
                        .or(in_state(AppState::Overworld)),
                ),
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
            broken_divinity::ui::gameover::check_player_death.run_if(in_state(AppState::Dungeon)),
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
        .add_systems(
            Update,
            broken_divinity::ui::inventory_panel::toggle_inventory
                .run_if(in_state(AppState::Dungeon).or(in_state(AppState::Colony))),
        )
        .add_systems(
            Update,
            broken_divinity::ui::journal_panel::toggle_journal.run_if(in_state(AppState::Dungeon)),
        )
        .add_systems(
            Update,
            broken_divinity::ui::stats_progression_panel::toggle_stats_progression.run_if(
                in_state(AppState::Dungeon)
                    .or(in_state(AppState::Colony))
                    .or(in_state(AppState::Overworld)),
            ),
        )
        .add_systems(
            Update,
            enter_overworld_from_colony.run_if(in_state(AppState::Colony)),
        )
        .add_systems(
            Update,
            handle_escape_to_menu
                .run_if(in_state(AppState::Colony).or(in_state(AppState::Overworld))),
        )
        .add_systems(
            Update,
            broken_divinity::core::save::handle_save_and_quit
                .run_if(in_state(AppState::Colony).or(in_state(AppState::Overworld))),
        );

    #[cfg(feature = "dev")]
    {
        app.add_plugins(RemotePlugin::default());
        app.add_plugins(RemoteHttpPlugin::default());
        app.add_plugins(BrpExtrasPlugin);
    }

    app.run();
}
