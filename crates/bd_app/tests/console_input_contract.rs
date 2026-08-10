//! Authoritative C1 console-input contracts.
//!
//! These tests intentionally exercise physical `KeyMessage` batches through
//! the registered production plugins. They do not call a reducer directly,
//! mutate the submission queue, or infer ownership from source text.

use bd_core::{
    BdSet,
    components::{Player, Position},
    session::RunSession,
    spatial::{GameMode, TransitionIntent},
};
use bd_test_support::foundation_content;
use bevy_app::{App, Update};
use bevy_ecs::{message::Messages, prelude::*};
use bevy_ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    event::KeyMessage,
};

fn key(code: KeyCode, kind: KeyEventKind) -> KeyMessage {
    KeyMessage(KeyEvent::new_with_kind(code, KeyModifiers::NONE, kind))
}

fn write_batch(app: &mut App, batch: impl IntoIterator<Item = KeyMessage>) {
    let mut messages = app.world_mut().resource_mut::<Messages<KeyMessage>>();
    for message in batch {
        messages.write(message);
    }
}

fn press(code: KeyCode) -> KeyMessage {
    key(code, KeyEventKind::Press)
}

fn full_runtime() -> App {
    let mut app = App::new();
    app.add_plugins(bd_core::BdFoundationPlugin);
    let content = foundation_content();
    app.insert_resource(bd_core::colony::stations::StationCatalog::new(
        content.stations.clone(),
    ));
    app.insert_resource(bd_core::factory::BlueprintCatalog::new(
        content.blueprints.clone(),
    ));
    app.insert_resource(content);
    app.add_plugins(bd_console::BdConsolePlugin);
    app.add_plugins(bd_tui::BdTuiPlugin);
    app
}

fn outpost_runtime() -> App {
    let mut app = full_runtime();
    app.world_mut()
        .resource_mut::<Messages<TransitionIntent>>()
        .write(TransitionIntent {
            target: GameMode::Outpost,
            node_id: None,
        });
    app.update();
    app.update();
    assert_eq!(*app.world().resource::<GameMode>(), GameMode::Outpost);
    app
}

fn player_position(app: &mut App) -> Position {
    let mut query = app.world_mut().query_filtered::<&Position, With<Player>>();
    *query
        .iter(app.world())
        .next()
        .expect("Foundation Outpost must contain a player")
}

#[derive(Resource, Default)]
struct ConsoleSubmissionAudit {
    observed: Vec<String>,
}

#[derive(Resource, Debug, Default)]
struct LegacyPendingAudit {
    quarantined: Vec<String>,
}

fn audit_console_submissions(
    mut commands: MessageReader<bd_console::ConsoleCommand>,
    mut audit: ResMut<ConsoleSubmissionAudit>,
) {
    audit
        .observed
        .extend(commands.read().map(|command| command.0.clone()));
}

fn quarantine_legacy_pending_path(
    mut console: ResMut<bd_console::ConsoleState>,
    mut audit: ResMut<LegacyPendingAudit>,
) {
    audit.quarantined = std::mem::take(&mut console.pending);
}

#[test]
fn physical_console_editing_uses_the_registered_production_reducer() {
    // Contract: CONSOLE-INPUT-001
    // Given: the console plugin is registered on the physical KeyMessage bus.
    // When: one ordered batch opens the console, types text, receives Repeat
    // and Release noise, and corrects the final character with Backspace.
    // Then: one production reducer owns the batch and leaves exactly `help`.
    // Must not change: Repeat/Release remain non-mutating and no direct test
    // queue/state simulation may substitute for the registered system.
    // Evidence layers: InputStateMachine, StateDiff.
    //
    // Implementation guidance:
    // - Reusable owner: one key reducer in bd_console owns console editing.
    // - Integration seam: BdConsolePlugin schedules that owner in BdSet::Input;
    //   the TUI may adapt whole-batch capture but must not copy editing rules.
    // - Preserve: ordered same-batch handling, printable ASCII, cursor updates,
    //   and Press-only mutation.
    // - Invalid shortcuts: manually setting ConsoleState, leaving the dormant
    //   reducer unscheduled, or retaining a second TUI editing match is not green.
    // - Closing evidence: run this exact test, the submission primary, every
    //   close-key support case, bd_console --lib, phase6_input, and the signed gate.
    let mut app = App::new();
    app.add_plugins(bd_core::BdCorePlugin);
    app.add_message::<KeyMessage>();
    app.add_plugins(bd_console::BdConsolePlugin);

    write_batch(
        &mut app,
        [
            press(KeyCode::Char('`')),
            press(KeyCode::Char('h')),
            press(KeyCode::Char('e')),
            key(KeyCode::Char('z'), KeyEventKind::Repeat),
            key(KeyCode::Char('y'), KeyEventKind::Release),
            press(KeyCode::Char('l')),
            press(KeyCode::Char('x')),
            press(KeyCode::Backspace),
            press(KeyCode::Char('p')),
        ],
    );
    app.update();

    let state = app.world().resource::<bd_console::ConsoleState>();
    assert!(
        state.open,
        "contract=CONSOLE-INPUT-001 checkpoint=open expected=true actual=false; \
         the registered console plugin did not own the physical batch"
    );
    assert_eq!(
        state.buffer, "help",
        "contract=CONSOLE-INPUT-001 checkpoint=ordered_edit expected=help actual={:?}; \
         Repeat/Release must be inert and Backspace must edit once",
        state.buffer
    );
    assert_eq!(state.cursor, 4);
    assert!(state.history.is_empty());
}

#[test]
fn one_physical_line_reaches_dispatch_exactly_once() {
    // Contract: CONSOLE-COMMAND-001
    // Given: the real Foundation + console + TUI plugin stack and an audit
    // reader between physical input and mutation dispatch.
    // When: one physical batch opens the console and submits `help`.
    // Then: exactly one typed ConsoleCommand reaches the seam, history records
    // one line, dispatch produces one help result, and no queue remains even
    // when the obsolete pre-Mutation pending path is quarantined.
    // Must not change: command parsing/output semantics or normal gameplay.
    // Evidence layers: InputStateMachine, Schedule, StateDiff, Workflow.
    //
    // Implementation guidance:
    // Adversarial influence table:
    // - Authoritative source: the ConsoleCommand emitted from physical Enter.
    // - Poisoned competitor: ConsoleState.pending written directly in Input.
    // - Derived outputs: typed audit, history, one help result, empty queue.
    // - Mixed-source shortcut: typed=[help], quarantined=[help], help_count=0.
    // - Independent observers: message audit, quarantine audit, final state.
    //
    // Implementation guidance:
    // - Reusable owner: the console reducer emits one ConsoleCommand and does
    //   not also write the legacy dispatch queue.
    // - Integration seam: one production bridge carries ConsoleCommand into
    //   the existing exclusive Mutation dispatcher before it runs.
    // - Preserve: same-frame ordered submission and an empty post-dispatch queue.
    // - Invalid shortcuts: retaining the reducer's parallel pending write,
    //   special-casing `help`, clearing this audit, or making the test call
    //   dispatch directly is not green.
    // - Closing evidence: pair with CONSOLE-INPUT-001 and both mode close matrices.
    let mut app = full_runtime();
    app.init_resource::<ConsoleSubmissionAudit>();
    app.init_resource::<LegacyPendingAudit>();
    app.add_systems(
        Update,
        (audit_console_submissions, quarantine_legacy_pending_path).in_set(BdSet::IntentCollection),
    );

    write_batch(
        &mut app,
        [
            press(KeyCode::Char('`')),
            press(KeyCode::Char('h')),
            press(KeyCode::Char('e')),
            press(KeyCode::Char('l')),
            press(KeyCode::Char('p')),
            press(KeyCode::Enter),
        ],
    );
    app.update();

    let audit = app.world().resource::<ConsoleSubmissionAudit>();
    let legacy = app.world().resource::<LegacyPendingAudit>();
    let state = app.world().resource::<bd_console::ConsoleState>();
    let help_count = state
        .output
        .iter()
        .filter(|line| line.contains("COMMANDS"))
        .count();
    assert_eq!(
        (
            audit.observed.clone(),
            legacy.quarantined.clone(),
            state.history.clone(),
            state.buffer.clone(),
            help_count,
            state.pending.clone(),
        ),
        (
            vec!["help".to_string()],
            Vec::<String>::new(),
            vec!["help".to_string()],
            String::new(),
            1,
            Vec::<String>::new(),
        ),
        "contract=CONSOLE-COMMAND-001 case=typed-causal-dispatch \
         expected=(typed=[help], legacy=[], history=[help], buffer='', help_count=1, pending=[]) \
         actual=(typed={:?}, legacy={:?}, history={:?}, buffer={:?}, help_count={}, pending={:?}); \
         a physical line must reach dispatch through ConsoleCommand rather than a parallel Input-stage queue write",
        audit.observed,
        legacy.quarantined,
        state.history,
        state.buffer,
        help_count,
        state.pending,
    );
}

fn assert_title_close_isolated(close_key: KeyCode, case_id: &str) {
    let mut app = full_runtime();
    {
        let mut console = app.world_mut().resource_mut::<bd_console::ConsoleState>();
        console.open = true;
        console.buffer = "partial".into();
        console.cursor = console.buffer.len();
    }
    write_batch(&mut app, [press(close_key)]);
    app.update();

    assert!(
        !app.world().resource::<bd_console::ConsoleState>().open,
        "contract=CONSOLE-INPUT-002 case={case_id} checkpoint=console_close"
    );
    assert_eq!(
        *app.world().resource::<GameMode>(),
        GameMode::Title,
        "contract=CONSOLE-INPUT-002 case={case_id} forbidden=title_begin"
    );
    assert_eq!(
        app.world()
            .resource::<bd_tui::screens::ScreenState>()
            .current,
        "title",
        "contract=CONSOLE-INPUT-002 case={case_id} forbidden=screen_transition"
    );
    assert!(
        !app.world()
            .resource::<bd_tui::commands::ApplicationExitRequest>()
            .0,
        "contract=CONSOLE-INPUT-002 case={case_id} forbidden=title_quit"
    );
}

#[test]
fn escape_close_is_consumed_before_title_routing() {
    // Contract: CONSOLE-INPUT-002 (primary)
    // Given: Title is active and an already-open console owns Escape.
    // When: one physical Escape closes the console.
    // Then: Title neither quits nor begins a run.
    // Must not change: the console still closes and clears transient editing.
    // Evidence layers: InputStateMachine, StateDiff, Workflow.
    //
    // Implementation guidance:
    // - Reusable owner: capture ownership for the whole physical batch before
    //   the reducer mutates `open`.
    // - Integration seam: gameplay routing consults that batch ownership, not
    //   the console's final open/closed value.
    // - Preserve: ordinary closed-console Escape remains the Title quit key.
    // - Invalid shortcuts: special-casing Title/Escape in gameplay routing or
    //   relying on reader order while both readers see the event is not green.
    // - Closing evidence: every support below must run independently.
    assert_title_close_isolated(KeyCode::Esc, "title-escape");
}

#[test]
fn backtick_close_is_consumed_before_title_routing() {
    // Supporting CONSOLE-INPUT-002: the toggle key is also owned by the batch
    // that began with the console open; Title's catch-all Begin path must not
    // observe it after the reducer closes the console.
    assert_title_close_isolated(KeyCode::Char('`'), "title-backtick");
}

#[test]
fn console_capture_is_explicitly_ordered_before_gameplay_routing() {
    // Supporting CONSOLE-INPUT-002.
    // Given: the real Foundation + console + TUI schedule has been initialized.
    // When: Bevy reports unresolved resource/message conflicts.
    // Then: capture_console_input and map_input_to_intents are both present and
    // are not an ambiguous pair; an explicit schedule dependency orders them.
    // Must not change: the existing physical close-key cases remain the
    // behavioral proof, and unrelated schedule ambiguities are outside C1.
    // Evidence layers: Schedule, InputStateMachine, Workflow.
    //
    // Implementation guidance:
    // - Reusable owner: the bd_console reducer remains the only editing owner.
    // - Integration seam: an explicit system/set edge orders capture before
    //   gameplay routing; insertion order and resource conflicts are not edges.
    // - Preserve: mode-agnostic whole-batch capture and closed-console input.
    // - Invalid shortcuts: requiring one plugin order, adding a Title/backtick
    //   exception, or making the no-op TUI guard the ordering proxy is not green.
    // - Closing evidence: run this independently, all physical close-key
    //   cases, both input neighbor suites, and the signed candidate gate.
    let mut app = full_runtime();
    app.update();

    let schedules = app.world().resource::<bevy_ecs::schedule::Schedules>();
    let schedule = schedules
        .get(Update)
        .expect("the production Update schedule must exist");
    use bevy_ecs::schedule::{IntoSystemSet, SystemSet};
    let capture_keys = schedule
        .graph()
        .systems_in_set(
            bd_console::input::capture_console_input
                .into_system_set()
                .intern(),
        )
        .expect("the registered console reducer must have a system type set");
    let unresolved_capture_conflicts = schedule
        .graph()
        .conflicting_systems()
        .iter()
        .filter(|(left, right, _)| capture_keys.contains(left) || capture_keys.contains(right))
        .map(|(left, right, conflicts)| (*left, *right, conflicts.len()))
        .collect::<Vec<_>>();
    let capture_ignores_all_ambiguities = capture_keys.iter().any(|key| {
        schedule
            .graph()
            .ambiguous_with_all
            .contains(&bevy_ecs::schedule::NodeId::System(*key))
    });

    assert_eq!(
        (
            capture_keys.len(),
            capture_ignores_all_ambiguities,
            unresolved_capture_conflicts.as_slice(),
        ),
        (1, false, &[][..]),
        "contract=CONSOLE-INPUT-002 case=explicit-capture-order \
         expected=(capture_systems=1, ignores_all=false, ambiguities=[]) \
         actual=(capture_systems={}, ignores_all={}, ambiguities={:?}); \
         the console reducer must have an explicit dependency before gameplay routing; \
         ambiguity suppression is not an ordering edge",
        capture_keys.len(),
        capture_ignores_all_ambiguities,
        unresolved_capture_conflicts,
    );
}

#[test]
fn escape_close_does_not_quit_or_mutate_outpost() {
    // Supporting CONSOLE-INPUT-002: Outpost Escape closes only the console.
    let mut app = outpost_runtime();
    app.world_mut()
        .resource_mut::<bd_console::ConsoleState>()
        .open = true;
    let turn_before = app.world().resource::<RunSession>().turn;
    let position_before = player_position(&mut app);

    write_batch(&mut app, [press(KeyCode::Esc)]);
    app.update();

    assert!(!app.world().resource::<bd_console::ConsoleState>().open);
    assert_eq!(*app.world().resource::<GameMode>(), GameMode::Outpost);
    assert_eq!(app.world().resource::<RunSession>().turn, turn_before);
    assert_eq!(player_position(&mut app), position_before);
    assert!(
        !app.world()
            .resource::<bd_tui::commands::ApplicationExitRequest>()
            .0,
        "contract=CONSOLE-INPUT-002 case=outpost-escape forbidden=quit"
    );
}

#[test]
fn backtick_close_does_not_reach_a_rebound_outpost_action() {
    // Supporting CONSOLE-INPUT-002: use a legal binding override so leakage of
    // an otherwise-unbound backtick has an observable gameplay consequence.
    let mut app = outpost_runtime();
    app.world_mut()
        .resource_mut::<bd_tui::commands::CommandBindings>()
        .bind(bd_tui::commands::UiCommand::Wait, KeyCode::Char('`'));
    app.world_mut()
        .resource_mut::<bd_console::ConsoleState>()
        .open = true;
    let turn_before = app.world().resource::<RunSession>().turn;
    let position_before = player_position(&mut app);

    write_batch(&mut app, [press(KeyCode::Char('`'))]);
    app.update();

    assert!(!app.world().resource::<bd_console::ConsoleState>().open);
    assert_eq!(
        app.world().resource::<RunSession>().turn,
        turn_before,
        "contract=CONSOLE-INPUT-002 case=outpost-backtick-rebound forbidden=wait"
    );
    assert_eq!(player_position(&mut app), position_before);
}

#[test]
fn closed_console_preserves_normal_gameplay_input() {
    // Preservation support for CONSOLE-INPUT-002: isolation is conditional.
    // A normal physical Press must still reach gameplay when the batch begins
    // with the console closed; Release remains inert.
    let mut app = outpost_runtime();
    let turn_before = app.world().resource::<RunSession>().turn;
    write_batch(
        &mut app,
        [
            key(KeyCode::Char('.'), KeyEventKind::Press),
            key(KeyCode::Char('.'), KeyEventKind::Release),
        ],
    );
    app.update();
    assert_eq!(
        app.world().resource::<RunSession>().turn,
        turn_before + 1,
        "closed-console input was over-captured instead of reaching gameplay once"
    );
}
