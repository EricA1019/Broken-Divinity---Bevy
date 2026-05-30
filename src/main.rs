use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
#[cfg(feature = "dev")]
use bevy::remote::RemotePlugin;
#[cfg(feature = "dev")]
use bevy_brp_extras::BrpExtrasPlugin;
#[cfg(feature = "ux-prototypes")]
use broken_divinity::ui::ux_unified_prototype::UnifiedPrototypePlugin;
use broken_divinity::core::escape::handle_escape_to_menu;
use broken_divinity::core::state::AppState;
use broken_divinity::core::turn::TurnPhase;
use broken_divinity::game::overworld::travel::enter_overworld_from_colony;

const WINDOW_TITLE_RUNTIME_AUTHORITY: &str = "Broken Divinity [Runtime UI]";
#[cfg(feature = "ux-prototypes")]
const WINDOW_TITLE_PROTOTYPE: &str = "Broken Divinity [Prototype UI]";
const UI_MODE_ENV_VAR: &str = "BD_UI_MODE";
const PROTOTYPE_MODE_VALUE: &str = "prototype";
const RUNTIME_AUTHORITY_MODE_MESSAGE: &str = "Runtime UI authority default";

fn print_launch_banner(mode: &str, command_hint: &str) {
    println!("================ BROKEN DIVINITY LAUNCH MODE ================");
    println!("Mode: {mode}");
    println!("Hint: {command_hint}");
    println!("=============================================================");
}

fn draw_launch_mode_badge(mut contexts: EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let mode = if prototype_mode_enabled() {
        "Launch Mode: Prototype UI (deprecated, opt-in)"
    } else if cfg!(feature = "dev") {
        "Launch Mode: Runtime UI authority (dev tooling enabled)"
    } else {
        "Launch Mode: Runtime UI authority"
    };

    egui::Area::new("launch_mode_badge".into())
        .anchor(egui::Align2::LEFT_TOP, [12.0, 12.0])
        .interactable(false)
        .show(ctx, |ui| {
            let frame = egui::Frame::new()
                .fill(egui::Color32::from_black_alpha(180))
                .corner_radius(egui::CornerRadius::same(4))
                .inner_margin(egui::Margin::symmetric(8, 6));

            frame.show(ui, |ui| {
                ui.label(
                    egui::RichText::new(mode)
                        .monospace()
                        .size(11.0)
                        .color(egui::Color32::from_rgb(255, 224, 160)),
                );
            });
        });
}

fn main() {
    if prototype_mode_enabled() {
        print_launch_banner(
            "Prototype UI enabled via BD_UI_MODE=prototype",
            "Use `BD_UI_MODE=prototype cargo run --bin broken_divinity --features ux-prototypes` only for prototype validation.",
        );
    } else if cfg!(feature = "dev") {
        print_launch_banner(
            "Runtime UI authority default (dev tooling enabled)",
            "Use `cargo run --bin broken_divinity` for runtime authority mode.",
        );
    } else {
        print_launch_banner(
            RUNTIME_AUTHORITY_MODE_MESSAGE,
            "Use `cargo run --bin broken_divinity` for runtime authority mode.",
        );
    }

    let mut app = App::new();
    configure_launch_app(&mut app);

    #[cfg(feature = "dev")]
    {
        app.add_plugins(RemotePlugin::default());
        app.add_plugins(BrpExtrasPlugin);
    }

    app.run();
}

fn prototype_mode_enabled() -> bool {
    std::env::var(UI_MODE_ENV_VAR)
        .map(|value| value.eq_ignore_ascii_case(PROTOTYPE_MODE_VALUE))
        .unwrap_or(false)
}

fn configure_launch_app(app: &mut App) {
    #[cfg(feature = "ux-prototypes")]
    if prototype_mode_enabled() {
        configure_prototype_app(app);
        return;
    }

    configure_runtime_authority_app(app);
}

#[cfg(feature = "ux-prototypes")]
fn configure_prototype_app(app: &mut App) {
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: WINDOW_TITLE_PROTOTYPE.to_string(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(EguiPlugin::default())
    .add_plugins(UnifiedPrototypePlugin)
    .add_systems(EguiPrimaryContextPass, draw_launch_mode_badge);
}

fn configure_runtime_authority_app(app: &mut App) {
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: WINDOW_TITLE_RUNTIME_AUTHORITY.to_string(),
            ..default()
        }),
        ..default()
    }))
        .add_plugins(TilemapPlugin)
        .add_plugins(EguiPlugin::default())
    .add_systems(EguiPrimaryContextPass, draw_launch_mode_badge)
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
        .init_resource::<broken_divinity::ui::inventory_panel::InventoryUiStatus>()
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
}
