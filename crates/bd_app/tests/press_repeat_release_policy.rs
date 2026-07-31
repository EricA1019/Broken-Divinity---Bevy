//! Press/Repeat/Release policy beyond Build — only a physical Press may
//! mutate gameplay; Repeat and Release are inert for every Foundation control.
//!
//! Contract: INPUT-POLICY-001
//! Authority: GDD "Minimum colony foundation"; docs/DECISIONS-TO-LOCK.md D-02;
//!            testing/FOUNDATION-REQUIREMENT-MAP.md section 2.
//!
//! Why this is intensive but not brittle:
//! - every row is a physical `KeyMessage` through the production
//!   BdFoundationPlugin + BdTuiPlugin stack (no synthetic intent writes);
//! - each row first proves its Press effect against a captured before-state
//!   (non-vacuity), then proves the same key as Repeat and as Release
//!   produces zero state change;
//! - observations are compact canonical state values (position, replay
//!   action count, screen, management, exit), not raw entity IDs or buffers;
//! - failure output names the contract, row, key kind, and expected/actual.

use bd_core::{
    colony::stations::StationCatalog,
    components::{Player, Position},
    session::RunSession,
    spatial::{GameMode, TransitionIntent},
};
use bd_test_support::foundation_content;
use bevy_app::App;
use bevy_ecs::{entity::Entity, message::Messages, query::With};
use bevy_ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    event::KeyMessage,
};

const CONTRACT: &str = "INPUT-POLICY-001";

// ---------------------------------------------------------------------------
// Production runtime
// ---------------------------------------------------------------------------

fn outpost_runtime() -> App {
    let mut app = App::new();
    app.add_plugins(bd_core::BdFoundationPlugin);
    let content = foundation_content();
    app.insert_resource(StationCatalog::new(content.stations.clone()));
    app.insert_resource(content);
    app.add_plugins(bd_tui::BdTuiPlugin);
    app.world_mut()
        .resource_mut::<Messages<TransitionIntent>>()
        .write(TransitionIntent {
            target: GameMode::Outpost,
            node_id: None,
        });
    app.update();
    app.update();
    app
}

fn player_entity(app: &mut App) -> Entity {
    let mut query = app.world_mut().query_filtered::<Entity, With<Player>>();
    query
        .iter(app.world())
        .next()
        .expect("Foundation player must exist")
}

fn send_kind(app: &mut App, key: KeyCode, kind: KeyEventKind) {
    app.world_mut()
        .resource_mut::<Messages<KeyMessage>>()
        .write(KeyMessage(KeyEvent::new_with_kind(
            key,
            KeyModifiers::NONE,
            kind,
        )));
}

// ---------------------------------------------------------------------------
// Canonical observable state
// ---------------------------------------------------------------------------

/// Every observable the policy covers, captured as a comparable value.
#[derive(Debug, Clone, PartialEq, Eq)]
struct BeforeState {
    position: Option<Position>,
    action_count: usize,
    screen: String,
    management: bool,
    exit: bool,
}

fn before_state(app: &mut App) -> BeforeState {
    let player = player_entity(app);
    BeforeState {
        position: app.world().get::<Position>(player).copied(),
        action_count: app.world().resource::<RunSession>().replay_intents.len(),
        screen: app
            .world()
            .resource::<bd_tui::screens::ScreenState>()
            .current
            .clone(),
        management: app
            .world()
            .resource::<bd_tui::view_models::StatsViewModel>()
            .management
            .is_some(),
        exit: app
            .world()
            .resource::<bd_tui::commands::ApplicationExitRequest>()
            .0,
    }
}

fn action_ids(app: &App) -> Vec<String> {
    app.world()
        .resource::<RunSession>()
        .replay_intents
        .iter()
        .map(|record| record.action_id.clone())
        .collect()
}

// ---------------------------------------------------------------------------
// Policy rows
// ---------------------------------------------------------------------------

struct PolicyCase {
    id: &'static str,
    key: KeyCode,
    /// After the key is pressed, assert the exact expected effect against the
    /// captured before-state.
    expect_press: fn(&mut App, &BeforeState, &str),
}

fn expect_exactly_one_action(app: &App, before: &BeforeState, case: &str, action_id: &str) {
    let ids = action_ids(app);
    assert_eq!(
        ids.len(),
        before.action_count + 1,
        "{CONTRACT} case={case} press expected exactly one resolved action"
    );
    assert_eq!(
        ids.last().map(String::as_str),
        Some(action_id),
        "{CONTRACT} case={case} press expected action `{action_id}`"
    );
}

const POLICY_CASES: &[PolicyCase] = &[
    PolicyCase {
        id: "move-east",
        key: KeyCode::Char('d'),
        expect_press: |app: &mut App, before, case| {
            let player = player_entity(app);
            let expected = before.position.map(|position| Position {
                x: position.x + 1,
                y: position.y,
            });
            assert_eq!(
                app.world().get::<Position>(player).copied(),
                expected,
                "{CONTRACT} case={case} press expected exactly one eastward move"
            );
        },
    },
    PolicyCase {
        id: "wait",
        key: KeyCode::Char('.'),
        expect_press: |app, before, case| {
            expect_exactly_one_action(app, before, case, "ability.wait")
        },
    },
    PolicyCase {
        id: "rest-until-next-day",
        key: KeyCode::Char('n'),
        expect_press: |app, before, case| {
            expect_exactly_one_action(app, before, case, "ability.rest_until_next_day")
        },
    },
    PolicyCase {
        id: "management",
        key: KeyCode::Char('c'),
        expect_press: |app, _before, case| {
            assert!(
                app.world()
                    .resource::<bd_tui::view_models::StatsViewModel>()
                    .management
                    .is_some(),
                "{CONTRACT} case={case} press expected the management menu to open"
            );
        },
    },
    PolicyCase {
        id: "help",
        key: KeyCode::Char('?'),
        expect_press: |app, before, case| {
            assert_ne!(
                before.screen, "help",
                "{CONTRACT} case={case} fixture must start outside Help"
            );
            assert_eq!(
                app.world()
                    .resource::<bd_tui::screens::ScreenState>()
                    .current,
                "help",
                "{CONTRACT} case={case} press expected the Help screen"
            );
        },
    },
    PolicyCase {
        id: "inventory",
        key: KeyCode::Char('i'),
        expect_press: |app, before, case| {
            assert_ne!(
                before.screen, "inventory",
                "{CONTRACT} case={case} fixture must start outside Inventory"
            );
            assert_eq!(
                app.world()
                    .resource::<bd_tui::screens::ScreenState>()
                    .current,
                "inventory",
                "{CONTRACT} case={case} press expected the Inventory screen"
            );
        },
    },
    PolicyCase {
        id: "quit",
        key: KeyCode::Char('q'),
        expect_press: |app, _before, case| {
            assert!(
                app.world()
                    .resource::<bd_tui::commands::ApplicationExitRequest>()
                    .0,
                "{CONTRACT} case={case} press expected one application exit request"
            );
        },
    },
];

// ---------------------------------------------------------------------------
// Matrix tests
// ---------------------------------------------------------------------------

/// A physical Repeat never mutates gameplay for any Foundation control.
#[test]
fn physical_repeat_never_mutates_any_foundation_control() {
    for case in POLICY_CASES {
        let mut app = outpost_runtime();
        let before = before_state(&mut app);

        send_kind(&mut app, case.key, KeyEventKind::Repeat);
        app.update();
        app.update();

        assert_eq!(
            before_state(&mut app),
            before,
            "{CONTRACT} case={} kind=Repeat must not mutate gameplay",
            case.id
        );
    }
}

/// A physical Release never mutates gameplay for any Foundation control.
#[test]
fn physical_release_never_mutates_any_foundation_control() {
    for case in POLICY_CASES {
        let mut app = outpost_runtime();
        let before = before_state(&mut app);

        send_kind(&mut app, case.key, KeyEventKind::Release);
        app.update();
        app.update();

        assert_eq!(
            before_state(&mut app),
            before,
            "{CONTRACT} case={} kind=Release must not mutate gameplay",
            case.id
        );
    }
}

/// Only the physical Press mutates, exactly once, for every Foundation
/// control. This anchors the two inert-kind tests and prevents them from
/// becoming vacuous.
#[test]
fn only_physical_press_mutates_exactly_once_for_every_foundation_control() {
    for case in POLICY_CASES {
        let mut app = outpost_runtime();
        let before = before_state(&mut app);

        send_kind(&mut app, case.key, KeyEventKind::Repeat);
        app.update();
        send_kind(&mut app, case.key, KeyEventKind::Release);
        app.update();
        assert_eq!(
            before_state(&mut app),
            before,
            "{CONTRACT} case={} Repeat+Release must leave state unchanged before Press",
            case.id
        );

        send_kind(&mut app, case.key, KeyEventKind::Press);
        app.update();
        app.update();

        assert_ne!(
            before_state(&mut app),
            before,
            "{CONTRACT} case={} Press must mutate the observable state (non-vacuous row)",
            case.id
        );
        (case.expect_press)(&mut app, &before, case.id);
    }
}
