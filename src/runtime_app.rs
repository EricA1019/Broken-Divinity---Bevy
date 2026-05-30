use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use crate::core::state::AppState;
use crate::core::turn::GameTime;
use crate::escape::{
    EscapeAction, EscapeContext, EscapeGuidanceEngine, EscapeGuidanceEvent, resolve_escape_action,
};
use crate::gamelog::GameLog;
use crate::objective_prompt::{InstructionEvent, ObjectivePromptEngine, ObjectivePromptPolicy};
use crate::primary_cta::{AppSurface, CtaPolicy, PrimaryCta};
use crate::runtime_flow::{FlowAction, FlowNode, RuntimeFlow};
use crate::save_recap::{SaveRecap, SaveRecapState, recap_for_state};
use crate::ui::menu::{MenuUiAction, MenuUiChoice, draw_main_menu, process_menu_action};
use crate::ui::runtime_action_language::RuntimeActionLanguage;

const DEFAULT_RUNTIME_SEED: u64 = 20_260_527;

pub fn build_runtime_app() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Broken Divinity".to_string(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(EguiPlugin::default());
    configure_runtime_composition(&mut app);
    app
}

pub fn run() {
    build_runtime_app().run();
}

pub fn configure_runtime_composition(app: &mut App) {
    app.insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.09)))
        .add_message::<AppExit>()
        .init_state::<AppState>()
        .insert_resource(GameLog::default())
        .insert_resource(GameTime::default())
        .insert_resource(MenuUiAction::default())
        .insert_resource(RuntimeFlow::default())
        .insert_resource(RuntimeShellState::default())
        .add_systems(Startup, setup_runtime_scene)
        .add_systems(
            Update,
            (
                process_runtime_shell_input,
                process_menu_action.run_if(in_state(AppState::Menu)),
                sync_runtime_flow_to_state,
            ),
        )
        .add_systems(EguiPrimaryContextPass, draw_main_menu)
        .add_systems(EguiPrimaryContextPass, draw_runtime_shell);
}

pub fn setup_runtime_scene(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Resource)]
struct RuntimeShellState {
    paused: bool,
    show_controls: bool,
    frame: u32,
    escape_guidance: EscapeGuidanceEngine,
    objective_prompt: ObjectivePromptEngine,
}

impl Default for RuntimeShellState {
    fn default() -> Self {
        Self {
            paused: false,
            show_controls: true,
            frame: 0,
            escape_guidance: EscapeGuidanceEngine::default(),
            objective_prompt: ObjectivePromptEngine::new(ObjectivePromptPolicy::default()),
        }
    }
}

pub fn flow_surface(node: FlowNode) -> Option<AppSurface> {
    match node {
        FlowNode::Menu => Some(AppSurface::Menu),
        FlowNode::Colony => Some(AppSurface::Colony),
        FlowNode::Overworld => Some(AppSurface::Overworld),
        FlowNode::Dungeon => None,
    }
}

pub fn app_state_for_flow(node: FlowNode) -> AppState {
    match node {
        FlowNode::Menu => AppState::Menu,
        FlowNode::Colony => AppState::Colony,
        FlowNode::Overworld => AppState::Overworld,
        FlowNode::Dungeon => AppState::Dungeon,
    }
}

pub fn flow_node_for_app_state(state: AppState) -> FlowNode {
    match state {
        AppState::Menu => FlowNode::Menu,
        AppState::Overworld => FlowNode::Overworld,
        AppState::Dungeon => FlowNode::Dungeon,
        AppState::Colony | AppState::Combat | AppState::GameOver => FlowNode::Colony,
    }
}

pub fn flow_primary_action(node: FlowNode) -> Option<FlowAction> {
    match node {
        FlowNode::Menu => Some(FlowAction::StartRun),
        FlowNode::Colony => Some(FlowAction::TravelToOverworld),
        FlowNode::Overworld => Some(FlowAction::EnterDungeon),
        FlowNode::Dungeon => Some(FlowAction::ReturnToColony),
    }
}

pub fn flow_primary_label(node: FlowNode) -> &'static str {
    RuntimeActionLanguage::flow_primary_label(node)
}

pub fn recap_for_flow(node: FlowNode) -> Option<SaveRecap> {
    match node {
        FlowNode::Menu => None,
        FlowNode::Colony => Some(recap_for_state(SaveRecapState::Colony)),
        FlowNode::Overworld => Some(recap_for_state(SaveRecapState::Overworld)),
        FlowNode::Dungeon => Some(recap_for_state(SaveRecapState::Dungeon)),
    }
}

fn process_runtime_shell_input(
    keys: Res<ButtonInput<KeyCode>>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut menu_action: ResMut<MenuUiAction>,
    mut flow: ResMut<RuntimeFlow>,
    mut shell: ResMut<RuntimeShellState>,
) {
    shell.frame = shell.frame.wrapping_add(1);

    let current_state = *app_state.get();

    if keys.just_pressed(KeyCode::F1) || keys.just_pressed(KeyCode::KeyH) {
        shell.show_controls = !shell.show_controls;
    }

    if keys.just_pressed(KeyCode::Escape) {
        let action = resolve_escape_action(EscapeContext {
            modal_open: shell.paused,
            can_pause: !matches!(flow.current(), FlowNode::Menu),
        });

        match action {
            EscapeAction::CloseModal => shell.paused = false,
            EscapeAction::PauseGame => shell.paused = true,
            EscapeAction::NoOp => {}
        }
    }

    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        if current_state == AppState::Menu {
            menu_action.0 = Some(MenuUiChoice::NewGame {
                seed: DEFAULT_RUNTIME_SEED,
            });
            return;
        }

        if let Some(action) = flow_primary_action(flow.current()) {
            if flow.apply(action).is_ok() {
                next_state.set(app_state_for_flow(flow.current()));
                shell.paused = false;
            }
        }
    }
}

fn sync_runtime_flow_to_state(app_state: Res<State<AppState>>, mut flow: ResMut<RuntimeFlow>) {
    let desired = flow_node_for_app_state(*app_state.get());
    if flow.current() != desired {
        flow.set_current(desired);
    }
}

fn draw_runtime_shell(
    mut contexts: EguiContexts,
    app_state: Res<State<AppState>>,
    mut flow: ResMut<RuntimeFlow>,
    mut shell: ResMut<RuntimeShellState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let current = flow.current();
    let paused = shell.paused;
    let frame = shell.frame;
    let surface = flow_surface(current);
    let cta_policy = CtaPolicy;
    let primary_cta = surface.map(|surface| cta_policy.primary_for(surface));
    let show_controls = shell.show_controls;
    let prompt = shell
        .objective_prompt
        .next(!matches!(current, FlowNode::Dungeon), !paused, frame);
    let escape_hint = shell.escape_guidance.guidance(EscapeContext {
        modal_open: paused,
        can_pause: !matches!(current, FlowNode::Menu),
    });

    if *app_state.get() == AppState::Menu {
        if show_controls {
            egui::Window::new("Controls")
                .anchor(egui::Align2::RIGHT_TOP, [-12.0, 12.0])
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Enter / Space: start a run from the menu");
                    ui.label("Esc: pause or close modal after the run starts");
                    ui.label("F1 or H: toggle controls panel");
                });
        }
        return;
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(24.0);
            ui.heading("Broken Divinity");
            ui.label("Recovery shell: Menu -> Colony -> Overworld -> Dungeon -> Colony");
            ui.add_space(16.0);

            ui.horizontal(|ui| {
                ui.label("Current state:");
                ui.monospace(format!("{current:?}"));
            });

            ui.horizontal(|ui| {
                ui.label("Primary CTA:");
                ui.monospace(flow_primary_label(current));
            });

            if let Some(cta) = primary_cta {
                ui.horizontal(|ui| {
                    ui.label("Policy:");
                    ui.monospace(match cta {
                        PrimaryCta::StartRun => "StartRun",
                        PrimaryCta::TravelToOverworld => "TravelToOverworld",
                        PrimaryCta::EnterDungeon => "EnterDungeon",
                    });
                });
            }

            if let Some(recap) = recap_for_flow(current) {
                ui.separator();
                ui.label("Save recap");
                ui.monospace(format!("{:#?}", recap));
            }

            ui.separator();
            ui.label(match prompt.kind {
                InstructionEvent::PrimaryShown => "Objective prompt: primary guidance active.",
                InstructionEvent::SecondaryShown => "Objective prompt: secondary guidance active.",
                InstructionEvent::TertiaryShown => "Objective prompt: tertiary guidance active.",
                InstructionEvent::SuppressedDuplicate => "Objective prompt: duplicate suppressed.",
            });

            if matches!(escape_hint, EscapeGuidanceEvent::ShowHint) {
                ui.label("Esc: pause or close the current modal.");
            }

            ui.add_space(16.0);

            if paused {
                ui.group(|ui| {
                    ui.label("Paused");
                    if ui.button("Resume").clicked() {
                        shell.paused = false;
                    }
                });
                return;
            }

            if let Some(action) = flow_primary_action(current) {
                if ui.button(flow_primary_label(current)).clicked() {
                    let _ = flow.apply(action);
                }
            }

            if matches!(current, FlowNode::Dungeon) {
                ui.label("Dungeon loop: return to the colony to regroup and prepare supplies.");
            }
        });
    });

    if show_controls {
        egui::Window::new("Controls")
            .anchor(egui::Align2::RIGHT_TOP, [-12.0, 12.0])
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Enter / Space: primary action");
                ui.label("Esc: pause or close modal");
                ui.label("F1 or H: toggle controls panel");
            });
    } else {
        egui::Area::new("controls_hint".into())
            .anchor(egui::Align2::LEFT_BOTTOM, [12.0, -12.0])
            .show(ctx, |ui| {
                ui.label("Press F1 for controls");
            });
    }
}
