//! Authoritative C2 final-composition contracts.
//!
//! Implementation-agent guidance:
//! - Fix the production ownership seam; never weaken these observers.
//! - The normal TUI draw must remain the one terminal draw owner.
//! - Compose the reusable console overlay after the normal screen inside that
//!   same draw closure. Do not add ordering between two terminal draws.
//! - Include only player-visible open-console state in invalidation. Hidden
//!   history, pending dispatch, and capture bookkeeping are not visual state.
//! - Run each named Red independently before trusting an aggregate result.

use super::*;
use bd_console::ConsoleState;
use bd_core::{components::Tile, spatial::GameMode};
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, layout::Rect};

fn shelter_map() -> MapViewModel {
    let width = 24;
    let height = 16;
    MapViewModel {
        width,
        height,
        tiles: vec![Tile::Floor; (width * height) as usize],
        ..Default::default()
    }
}

fn draw_fixture(terminal: &mut Terminal<TestBackend>, console: Option<&ConsoleState>) -> Buffer {
    let screens = default_screen_registry();
    let widgets = default_widget_registry();
    let map = shelter_map();
    let stats = StatsViewModel::default();
    let log = LogViewModel {
        entries: vec![view_models::LogEntryVm {
            message: "UNDERLAY-C2-POISON".into(),
            level: LogLevel::Info,
        }],
    };
    let actions = ActionListViewModel::default();
    let container = ContainerViewModel::default();
    let event = EventViewModel::default();
    let help = HelpViewModel::default();
    let symbols = SymbolRegistry::phase5_defaults();
    let theme = ThemeRegistry::phase5_defaults();
    let bindings = commands::CommandBindings::default();
    let definition = screens.get("outpost").expect("outpost fixture must exist");
    let data = UiFrameData {
        definition,
        widgets: &widgets,
        map: &map,
        stats: &stats,
        log: &log,
        actions: &actions,
        container: &container,
        event: &event,
        help: &help,
        symbols: &symbols,
        theme: &theme,
        bindings: &bindings,
        mode: GameMode::Outpost,
        interaction: commands::InteractionMode::Normal,
        turn: 7,
        day: 2,
    };

    terminal
        .draw(|frame| render_final_frame(frame, &data, console))
        .expect("C2 fixture must reach the production final compositor");
    terminal.backend().buffer().clone()
}

fn render_fixture(width: u16, height: u16, console: Option<&ConsoleState>) -> Buffer {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("C2 terminal must initialize");
    draw_fixture(&mut terminal, console)
}

fn text_in(buffer: &Buffer, area: Rect) -> String {
    (area.y..area.y + area.height)
        .map(|y| {
            (area.x..area.x + area.width)
                .map(|x| {
                    buffer
                        .cell((x, y))
                        .expect("C2 observation must remain inside the terminal")
                        .symbol()
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn border_violations(buffer: &Buffer, area: Rect) -> Vec<String> {
    let left = area.x;
    let right = area.x + area.width - 1;
    let top = area.y;
    let bottom = area.y + area.height - 1;
    let mut violations = Vec::new();

    for (position, expected) in [
        ((left, top), "┌"),
        ((right, top), "┐"),
        ((left, bottom), "└"),
        ((right, bottom), "┘"),
    ] {
        let actual = buffer.cell(position).map(|cell| cell.symbol());
        if actual != Some(expected) {
            violations.push(format!(
                "position={position:?} expected={expected:?} actual={actual:?}"
            ));
        }
    }
    for x in left + 1..right {
        let actual = buffer.cell((x, bottom)).map(|cell| cell.symbol());
        if actual != Some("─") {
            violations.push(format!(
                "position=({x}, {bottom}) expected=\"─\" actual={actual:?}"
            ));
        }
    }
    for y in top + 1..bottom {
        for x in [left, right] {
            let actual = buffer.cell((x, y)).map(|cell| cell.symbol());
            if actual != Some("│") {
                violations.push(format!(
                    "position=({x}, {y}) expected=\"│\" actual={actual:?}"
                ));
            }
        }
    }
    violations
}

#[test]
fn open_console_survives_authoritative_final_composition_at_supported_profiles() {
    // Primary: CONSOLE-RENDER-001.
    // Current intentional Red: the final TUI compositor ignores ConsoleState,
    // so CONSOLE/output/prompt/buffer are all absent at both profiles.
    let console = ConsoleState {
        open: true,
        buffer: "typed-c2-probe".into(),
        cursor: "typed-c2-probe".len(),
        output: vec!["OK: C2-FINAL-OUTPUT".into()],
        ..Default::default()
    };

    for (width, height) in [(80, 24), (60, 20)] {
        let buffer = render_fixture(width, height, Some(&console));
        let overlay = bd_console::render::console_overlay_area(buffer.area);
        let text = text_in(&buffer, overlay);
        let border = border_violations(&buffer, overlay);
        let required = ["CONSOLE", "OK: C2-FINAL-OUTPUT", "> typed-c2-probe"];
        let missing = required
            .into_iter()
            .filter(|needle| !text.contains(needle))
            .collect::<Vec<_>>();

        assert!(
            missing.is_empty() && border.is_empty(),
            "contract=CONSOLE-RENDER-001 case={width}x{height} \
             expected=final_overlay_with_output_prompt_buffer_and_complete_border \
             missing={missing:?} border_violations={border:?} overlay={overlay:?}\n{text}"
        );
    }
}

#[test]
fn visible_console_state_invalidates_the_authoritative_frame_once() {
    // Supporting: CONSOLE-RENDER-001.
    // The exact hash value is deliberately unconstrained. Only semantic
    // equality/difference is authoritative, which keeps this strict without
    // coupling the test to a hash implementation.
    let base = 0xC2_u64;
    let closed_a = ConsoleState::default();
    let closed_b = ConsoleState {
        buffer: "hidden edit".into(),
        output: vec!["hidden output".into()],
        history: vec!["hidden history".into()],
        pending: vec!["hidden pending".into()],
        ..Default::default()
    };

    let open_a = ConsoleState {
        open: true,
        buffer: "a".into(),
        cursor: 1,
        output: vec!["line-a".into()],
        ..Default::default()
    };
    let mut open_buffer = open_a.clone();
    open_buffer.buffer = "b".into();
    open_buffer.cursor = 1;
    let mut open_output = open_a.clone();
    open_output.output = vec!["line-b".into()];
    let mut open_hidden = open_a.clone();
    open_hidden.history.push("not rendered".into());
    open_hidden.pending.push("not rendered".into());
    open_hidden.batch_capture_active = true;

    let hash = |state: &ConsoleState| visible_console_fingerprint(base, Some(state));
    let no_console_hash = visible_console_fingerprint(base, None);
    let closed_hash = hash(&closed_a);
    let open_hash = hash(&open_a);
    let observations = [
        ("no-console-vs-closed", no_console_hash, closed_hash, true),
        ("closed-hidden-state", closed_hash, hash(&closed_b), true),
        ("open-transition", closed_hash, open_hash, false),
        ("open-buffer", open_hash, hash(&open_buffer), false),
        ("open-output", open_hash, hash(&open_output), false),
        ("open-hidden-state", open_hash, hash(&open_hidden), true),
    ];
    let violations = observations
        .into_iter()
        .filter_map(|(case, left, right, should_match)| {
            (should_match != (left == right)).then_some(format!(
                "case={case} expected_equal={should_match} left={left} right={right}"
            ))
        })
        .collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "contract=CONSOLE-RENDER-001 expected=visible_console_state_participates_once \
         violations={violations:?}"
    );
}

#[test]
fn closed_console_matches_clean_canvas_and_styles_at_supported_profiles() {
    // Primary preservation row: CONSOLE-RENDER-002.
    let closed = ConsoleState::default();
    for (width, height) in [(80, 24), (60, 20)] {
        let clean = render_fixture(width, height, None);
        let composed_closed = render_fixture(width, height, Some(&closed));
        assert_eq!(
            composed_closed, clean,
            "contract=CONSOLE-RENDER-002 case={width}x{height} \
             expected=identical_clean_canvas_and_resolved_styles_after_closed_composition"
        );
    }
}

#[test]
fn open_resize_close_returns_to_clean_authoritative_output() {
    // Transition preservation: CONSOLE-RENDER-002. Reuse one terminal so stale
    // cells cannot hide behind fresh-backend comparisons.
    let open = ConsoleState {
        open: true,
        buffer: "resize-c2".into(),
        cursor: "resize-c2".len(),
        output: vec!["OK: before resize".into()],
        ..Default::default()
    };
    let closed = ConsoleState::default();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("C2 terminal must initialize");

    let opened = draw_fixture(&mut terminal, Some(&open));
    assert!(
        text_in(
            &opened,
            bd_console::render::console_overlay_area(opened.area)
        )
        .contains("CONSOLE"),
        "contract=CONSOLE-RENDER-002 workflow_step=open expected=visible_overlay"
    );

    terminal.backend_mut().resize(60, 20);
    terminal
        .resize(Rect::new(0, 0, 60, 20))
        .expect("supported-profile resize must succeed");
    let resized_open = draw_fixture(&mut terminal, Some(&open));
    assert!(
        text_in(
            &resized_open,
            bd_console::render::console_overlay_area(resized_open.area)
        )
        .contains("CONSOLE"),
        "contract=CONSOLE-RENDER-002 workflow_step=resize expected=visible_overlay"
    );
    let after_close = draw_fixture(&mut terminal, Some(&closed));
    let clean = render_fixture(60, 20, None);
    assert_eq!(
        after_close, clean,
        "contract=CONSOLE-RENDER-002 workflow=open_80x24_resize_60x20_close \
         expected=clean_authoritative_canvas_and_styles"
    );
}

#[test]
fn multi_line_command_output_preserves_logical_rows() {
    // Supporting CONSOLE-RENDER-001: embedded newlines in one output entry
    // (e.g. `stats`) must render as separate rows instead of collapsing into
    // one wrapped paragraph.
    let console = ConsoleState {
        open: true,
        buffer: String::new(),
        cursor: 0,
        output: vec!["day: 0\nturn: 1".into()],
        ..Default::default()
    };
    let buffer = render_fixture(80, 24, Some(&console));
    let overlay = bd_console::render::console_overlay_area(buffer.area);
    let text = text_in(&buffer, overlay);
    let lines: Vec<&str> = text.split('\n').collect();
    let day_row = lines.iter().position(|line| line.contains("day: 0"));
    let turn_row = lines.iter().position(|line| line.contains("turn: 1"));
    assert!(
        day_row.is_some() && turn_row.is_some() && day_row != turn_row,
        "contract=CONSOLE-RENDER-001 case=multi-line-output \
         expected=separate-rows day={day_row:?} turn={turn_row:?}\n{text}"
    );
}
