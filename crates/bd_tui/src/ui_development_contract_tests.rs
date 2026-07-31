//! Red-first player-facing contracts for the Foundation UI improvement plan.
//!
//! These tests deliberately separate layout, semantic content, resolved
//! style, and transition evidence. A failing test in this module describes a
//! UI development target; it must not be weakened merely to restore a green
//! aggregate test count.

use super::*;
use bd_core::{
    components::{Position, Tile},
    gamelog::LogLevel,
    spatial::GameMode,
};
use ratatui::{
    Terminal,
    backend::TestBackend,
    buffer::Buffer,
    layout::{Rect, Size},
    style::{Color, Modifier, Style},
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct CellObservation {
    x: u16,
    y: u16,
    symbol: String,
    foreground: Color,
    background: Color,
    modifier: Modifier,
}

fn observe(buffer: &Buffer) -> Vec<CellObservation> {
    let area = buffer.area;
    (area.y..area.y + area.height)
        .flat_map(|y| {
            (area.x..area.x + area.width).map(move |x| {
                let cell = buffer
                    .cell((x, y))
                    .expect("observation coordinate must be inside the buffer");
                CellObservation {
                    x,
                    y,
                    symbol: cell.symbol().to_owned(),
                    foreground: cell.fg,
                    background: cell.bg,
                    modifier: cell.modifier,
                }
            })
        })
        .collect()
}

fn shelter_map() -> MapViewModel {
    let width = 40;
    let height = 30;
    MapViewModel {
        width,
        height,
        tiles: vec![Tile::Floor; (width * height) as usize],
        player_pos: Some(Position { x: 1, y: 1 }),
        visuals: vec![view_models::MapVisualVm {
            position: Position { x: 1, y: 1 },
            token: visual::VisualToken::Player,
            glyph: None,
        }],
        ..Default::default()
    }
}

#[allow(clippy::too_many_arguments)]
fn render_buffer(
    screen: &str,
    width: u16,
    height: u16,
    mode: GameMode,
    map: &MapViewModel,
    stats: &StatsViewModel,
    log: &LogViewModel,
    actions: &ActionListViewModel,
) -> Buffer {
    let screens = default_screen_registry();
    let widgets = default_widget_registry();
    let container = ContainerViewModel::default();
    let event = EventViewModel::default();
    let help = HelpViewModel::default();
    let symbols = SymbolRegistry::phase5_defaults();
    let theme = ThemeRegistry::phase5_defaults();
    let bindings = commands::CommandBindings::default();
    let definition = screens.get(screen).expect("screen fixture must exist");
    let interaction = frame_interaction(
        mode,
        map.build_menu.is_some() || map.build_ghost.is_some(),
        stats.management.as_ref().map(|menu| menu.kind),
    );
    let data = UiFrameData {
        definition,
        widgets: &widgets,
        map,
        stats,
        log,
        actions,
        container: &container,
        event: &event,
        help: &help,
        symbols: &symbols,
        theme: &theme,
        bindings: &bindings,
        mode,
        interaction,
        turn: 0,
        day: stats.day,
    };
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("test terminal must initialize");
    terminal
        .draw(|frame| render_ui_frame(frame, &data))
        .expect("test frame must render");
    terminal.backend().buffer().clone()
}

fn buffer_text(buffer: &Buffer) -> String {
    let area = buffer.area;
    (area.y..area.y + area.height)
        .map(|y| {
            (area.x..area.x + area.width)
                .map(|x| {
                    buffer
                        .cell((x, y))
                        .expect("text coordinate must be inside the buffer")
                        .symbol()
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn default_actions() -> ActionListViewModel {
    ActionListViewModel::default()
}

fn unique_symbol(buffer: &Buffer, symbol: &str) -> CellObservation {
    let matches = observe(buffer)
        .into_iter()
        .filter(|cell| cell.symbol == symbol)
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "fixture expected one `{symbol}` cell; matches={matches:?}\n{}",
        buffer_text(buffer)
    );
    matches.into_iter().next().expect("one match was asserted")
}

fn normalized_text(buffer: &Buffer) -> String {
    buffer_text(buffer)
        .chars()
        .map(|character| {
            if matches!(character, '│' | '┌' | '┐' | '└' | '┘' | '─') {
                ' '
            } else {
                character
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalized_panel_text(buffer: &Buffer, screen: &str, panel_id: &str) -> String {
    let area = buffer.area;
    let registry = default_screen_registry();
    let canonical = registry.get(screen).expect("screen fixture must exist");
    let definition = if commands::terminal_layout(area.width, area.height)
        == commands::TerminalLayout::Compact
    {
        screens::compact_screen_definition(canonical)
    } else {
        canonical.clone()
    };
    let content = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(3));
    let panel = compute_panel_rects(&definition, content)
        .into_iter()
        .find_map(|(id, rect)| (id == panel_id).then_some(rect))
        .unwrap_or_else(|| panic!("screen `{screen}` has no `{panel_id}` panel"));
    (panel.y..panel.y + panel.height)
        .flat_map(|y| {
            (panel.x..panel.x + panel.width).map(move |x| {
                buffer
                    .cell((x, y))
                    .expect("panel coordinate must be inside the buffer")
                    .symbol()
            })
        })
        .collect::<String>()
        .chars()
        .map(|character| {
            if matches!(character, '│' | '┌' | '┐' | '└' | '┘' | '─') {
                ' '
            } else {
                character
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn visual_observation_detects_glyph_foreground_modifier_and_geometry_changes() {
    // Contract: VISUAL-OBSERVATION-001
    let mut baseline = Buffer::empty(Rect::new(0, 0, 3, 2));
    baseline.set_string(0, 0, "A", Style::default().fg(Color::White));

    let mut glyph = baseline.clone();
    glyph.set_string(0, 0, "B", Style::default().fg(Color::White));
    assert_ne!(
        observe(&baseline),
        observe(&glyph),
        "glyph-only diff was lost"
    );

    let mut foreground = baseline.clone();
    foreground.set_string(0, 0, "A", Style::default().fg(Color::Red));
    assert_ne!(
        observe(&baseline),
        observe(&foreground),
        "foreground-only diff was lost"
    );

    let mut modifier = baseline.clone();
    modifier.set_string(
        0,
        0,
        "A",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    assert_ne!(
        observe(&baseline),
        observe(&modifier),
        "modifier-only diff was lost"
    );

    let larger = Buffer::empty(Rect::new(0, 0, 4, 2));
    assert_ne!(
        observe(&baseline),
        observe(&larger),
        "geometry diff was lost"
    );
}

#[test]
fn identical_fixture_has_identical_canvas_and_resolved_styles() {
    // Supporting deterministic evidence for VISUAL-OBSERVATION-001.
    let map = shelter_map();
    let stats = StatsViewModel::default();
    let log = LogViewModel::default();
    let actions = default_actions();
    let first = render_buffer(
        "outpost",
        80,
        24,
        GameMode::Outpost,
        &map,
        &stats,
        &log,
        &actions,
    );
    let second = render_buffer(
        "outpost",
        80,
        24,
        GameMode::Outpost,
        &map,
        &stats,
        &log,
        &actions,
    );

    assert_eq!(
        observe(&first),
        observe(&second),
        "same-state rendering is not deterministic"
    );
}

#[test]
fn supported_outpost_panel_rectangles_never_overlap() {
    // Supporting geometry evidence for VISUAL-LAYOUT-001.
    for (width, height) in [(80_u16, 24_u16), (60, 20)] {
        let registry = default_screen_registry();
        let canonical = registry.get("outpost").expect("outpost must exist");
        let definition =
            if commands::terminal_layout(width, height) == commands::TerminalLayout::Compact {
                screens::compact_screen_definition(canonical)
            } else {
                canonical.clone()
            };
        let content = Rect::new(0, 0, width, height - 3);
        let rectangles = compute_panel_rects(&definition, content);

        for (left_index, (left_id, left)) in rectangles.iter().enumerate() {
            assert!(
                left.width > 0 && left.height > 0,
                "contract=VISUAL-LAYOUT-001 case={width}x{height} panel={left_id} \
                 has unusable geometry {left:?}"
            );
            for (right_id, right) in rectangles.iter().skip(left_index + 1) {
                let horizontal = left.x < right.x + right.width && right.x < left.x + left.width;
                let vertical = left.y < right.y + right.height && right.y < left.y + left.height;
                assert!(
                    !(horizontal && vertical),
                    "contract=VISUAL-LAYOUT-001 case={width}x{height} \
                     panels overlap: {left_id}={left:?}, {right_id}={right:?}"
                );
            }
        }
    }
}

#[test]
fn outpost_map_is_the_largest_interactive_panel_at_supported_profiles() {
    // Contract: VISUAL-LAYOUT-001
    for (width, height) in [(80_u16, 24_u16), (60, 20)] {
        let registry = default_screen_registry();
        let canonical = registry.get("outpost").expect("outpost must exist");
        let definition =
            if commands::terminal_layout(width, height) == commands::TerminalLayout::Compact {
                screens::compact_screen_definition(canonical)
            } else {
                canonical.clone()
            };
        let rectangles = compute_panel_rects(&definition, Rect::new(0, 0, width, height - 3));
        let map_area = rectangles
            .iter()
            .find(|(id, _)| id == "map")
            .map(|(_, rect)| u32::from(rect.width) * u32::from(rect.height))
            .expect("outpost must contain a map");
        let largest_other = rectangles
            .iter()
            .filter(|(id, _)| id != "map")
            .map(|(_, rect)| u32::from(rect.width) * u32::from(rect.height))
            .max()
            .unwrap_or_default();

        assert!(
            map_area > largest_other,
            "contract=VISUAL-LAYOUT-001 case={width}x{height} \
             expected physical shelter map to own the largest interactive area; \
             map_area={map_area}, largest_other={largest_other}, panels={rectangles:?}"
        );
    }
}

#[test]
fn offscreen_assignment_names_target_and_distance_at_supported_profiles() {
    // Contract: VISUAL-VIEWPORT-005
    let mut map = shelter_map();
    map.assigned_targets = vec![Position { x: 30, y: 29 }];
    map.assigned_target_details = vec![view_models::AssignedTargetVm {
        position: Position { x: 30, y: 29 },
        label: "Water Source".into(),
        survivor: "Mara".into(),
    }];
    for (width, height) in [(80, 24), (60, 20)] {
        let buffer = render_buffer(
            "outpost",
            width,
            height,
            GameMode::Outpost,
            &map,
            &StatsViewModel::default(),
            &LogViewModel::default(),
            &default_actions(),
        );
        let text = normalized_text(&buffer);
        for required in ["Water Source", "57 tiles"] {
            assert!(
                text.contains(required),
                "contract=VISUAL-VIEWPORT-005 case={width}x{height} \
                 fixture=offscreen-water-target missing `{required}`; \
                 expected direction, target name, and distance\n{}",
                buffer_text(&buffer)
            );
        }
    }
}

#[test]
fn valid_and_invalid_build_previews_differ_without_color() {
    // Contract: VISUAL-BUILD-005
    let mut valid_map = shelter_map();
    valid_map.build_ghost = Some((Position { x: 2, y: 1 }, '¤'));
    valid_map.build_placement = Some(view_models::BuildPlacementVm {
        label: "Workshop".into(),
        supply_cost: 4,
        effect: "Produces refined Materials".into(),
    });
    let valid = render_buffer(
        "outpost",
        80,
        24,
        GameMode::Outpost,
        &valid_map,
        &StatsViewModel::default(),
        &LogViewModel::default(),
        &default_actions(),
    );

    let mut invalid_map = valid_map.clone();
    invalid_map.build_ghost_denial = Some("Occupied by Basic Processing".into());
    let invalid = render_buffer(
        "outpost",
        80,
        24,
        GameMode::Outpost,
        &invalid_map,
        &StatsViewModel::default(),
        &LogViewModel::default(),
        &default_actions(),
    );
    let valid_cell = unique_symbol(&valid, "¤");
    let invalid_cell = unique_symbol(&invalid, "!");

    assert_ne!(
        (&valid_cell.symbol, valid_cell.modifier),
        (&invalid_cell.symbol, invalid_cell.modifier),
        "contract=VISUAL-BUILD-005 fixture=valid-vs-invalid-preview \
         color-independent preview semantics are identical; \
         valid={valid_cell:?}, invalid={invalid_cell:?}"
    );
}

#[test]
fn unaffordable_build_selection_explains_the_exact_shortage() {
    // Contract: VISUAL-BUILD-006
    let mut map = shelter_map();
    map.build_menu = Some(view_models::BuildMenuVm {
        options: vec![("Workshop".into(), 4, "Produces refined Materials".into())],
        selected: 0,
        available_supplies: 1,
    });
    for (width, height) in [(80, 24), (60, 20)] {
        let buffer = render_buffer(
            "outpost",
            width,
            height,
            GameMode::Outpost,
            &map,
            &StatsViewModel::default(),
            &LogViewModel::default(),
            &default_actions(),
        );
        assert!(
            normalized_text(&buffer).contains("Need 3 more Supplies"),
            "contract=VISUAL-BUILD-006 case={width}x{height} \
             expected exact unavailable reason `Need 3 more Supplies`\n{}",
            buffer_text(&buffer)
        );
    }
}

#[test]
fn task_management_exposes_survivor_task_and_confirm_stages() {
    // Contract: VISUAL-MGMT-003
    let stats = StatsViewModel {
        management: Some(view_models::ManagementMenuVm {
            kind: view_models::ManagementMenuKind::TaskAssignment,
            survivors: vec!["Mara — Idle".into()],
            tasks: vec!["1. Gather Supplies".into()],
            selected_survivor: Some(0),
            selected_task: Some(0),
            resources: "Sup 4  Mat 0  Plant 0  Faith 0".into(),
            forecast: "Next worker: none | Next day: upkeep -3".into(),
        }),
        ..Default::default()
    };
    for (width, height) in [(80, 24), (60, 20)] {
        let buffer = render_buffer(
            "outpost",
            width,
            height,
            GameMode::Outpost,
            &shelter_map(),
            &stats,
            &LogViewModel::default(),
            &default_actions(),
        );
        let text = normalized_text(&buffer);
        for required in ["1 Survivor", "2 Task", "3 Confirm"] {
            assert!(
                text.contains(required),
                "contract=VISUAL-MGMT-003 case={width}x{height} \
                 missing workflow stage `{required}`\n{}",
                buffer_text(&buffer)
            );
        }
    }
}

#[test]
fn station_staffing_exposes_survivor_station_recipe_and_confirm_stages() {
    // Contract: VISUAL-MGMT-004
    let stats = StatsViewModel {
        management: Some(view_models::ManagementMenuVm {
            kind: view_models::ManagementMenuKind::StationStaffing,
            survivors: vec!["Mara — Idle".into()],
            tasks: vec!["1. Refine Timber — Raw Timber → Refined Materials".into()],
            selected_survivor: Some(0),
            selected_task: Some(0),
            resources: "Sup 4  Mat 0  Plant 0  Faith 0".into(),
            forecast: "Next worker: Refined Materials in 2 turns".into(),
        }),
        ..Default::default()
    };
    for (width, height) in [(80, 24), (60, 20)] {
        let buffer = render_buffer(
            "outpost",
            width,
            height,
            GameMode::Outpost,
            &shelter_map(),
            &stats,
            &LogViewModel::default(),
            &default_actions(),
        );
        let text = normalized_text(&buffer);
        for required in ["1 Survivor", "2 Station", "3 Recipe", "4 Confirm"] {
            assert!(
                text.contains(required),
                "contract=VISUAL-MGMT-004 case={width}x{height} \
                 missing workflow stage `{required}`\n{}",
                buffer_text(&buffer)
            );
        }
    }
}

#[test]
fn blocked_worker_has_a_distinct_resolved_style_from_working_worker() {
    // Contract: VISUAL-LANGUAGE-004
    let mut map = shelter_map();
    map.visuals.extend([
        view_models::MapVisualVm {
            position: Position { x: 2, y: 1 },
            token: visual::VisualToken::WorkerWorking,
            glyph: Some('ŵ'),
        },
        view_models::MapVisualVm {
            position: Position { x: 3, y: 1 },
            token: visual::VisualToken::WorkerBlocked,
            glyph: Some('ƀ'),
        },
    ]);
    let buffer = render_buffer(
        "outpost",
        80,
        24,
        GameMode::Outpost,
        &map,
        &StatsViewModel::default(),
        &LogViewModel::default(),
        &default_actions(),
    );
    let working = unique_symbol(&buffer, "ŵ");
    let blocked = unique_symbol(&buffer, "ƀ");

    assert_ne!(
        (working.foreground, working.background, working.modifier),
        (blocked.foreground, blocked.background, blocked.modifier),
        "contract=VISUAL-LANGUAGE-004 fixture=working-vs-blocked \
         resolved styles are identical; working={working:?}, blocked={blocked:?}"
    );
}

#[test]
fn decisive_warning_survives_routine_log_overflow() {
    // Contract: VISUAL-FEEDBACK-001
    let mut entries = vec![view_models::LogEntryVm {
        message: "Cannot build here — shelter egress would be blocked".into(),
        level: LogLevel::Warn,
    }];
    entries.extend((1..=20).map(|turn| view_models::LogEntryVm {
        message: format!("Routine worker movement {turn}"),
        level: LogLevel::Info,
    }));
    let buffer = render_buffer(
        "outpost",
        80,
        24,
        GameMode::Outpost,
        &shelter_map(),
        &StatsViewModel::default(),
        &LogViewModel { entries },
        &default_actions(),
    );

    assert!(
        normalized_text(&buffer).contains("Cannot build here"),
        "contract=VISUAL-FEEDBACK-001 fixture=warning-plus-routine-overflow \
         decisive warning was buried before the player could act\n{}",
        buffer_text(&buffer)
    );
}

#[test]
fn rendered_dungeon_denials_explain_attack_pickup_and_extraction() {
    // Supporting rendered evidence for VISUAL-DUNGEON-001.
    let actions = ActionListViewModel {
        actions: vec![
            view_models::ActionItemVm {
                label: "Attack".into(),
                key_hint: "f".into(),
                enabled: false,
                denial_reason: Some("No adjacent target".into()),
            },
            view_models::ActionItemVm {
                label: "Pickup".into(),
                key_hint: "g".into(),
                enabled: false,
                denial_reason: Some("No loot here".into()),
            },
            view_models::ActionItemVm {
                label: "Extract".into(),
                key_hint: ">".into(),
                enabled: false,
                denial_reason: Some("Reach the exit".into()),
            },
        ],
    };
    for (width, height) in [(80, 24), (60, 20)] {
        let buffer = render_buffer(
            "combat",
            width,
            height,
            GameMode::Tactical,
            &shelter_map(),
            &StatsViewModel::default(),
            &LogViewModel::default(),
            &actions,
        );
        let text = normalized_panel_text(&buffer, "combat", "actions");
        for required in ["No adjacent target", "No loot here", "Reach the exit"] {
            assert!(
                text.contains(required),
                "contract=VISUAL-DUNGEON-001 case={width}x{height} \
                 rendered action panel hid `{required}`\n{}",
                buffer_text(&buffer)
            );
        }
    }
}

#[test]
fn dungeon_status_distinguishes_carried_loot_and_extraction_readiness() {
    // Contract: VISUAL-DUNGEON-001
    let stats = StatsViewModel {
        hp_current: 8,
        hp_max: 10,
        ap_current: 2,
        ap_max: 3,
        carried_loot: 2,
        extraction_ready: true,
        ..Default::default()
    };
    let buffer = render_buffer(
        "combat",
        80,
        24,
        GameMode::Tactical,
        &shelter_map(),
        &stats,
        &LogViewModel::default(),
        &default_actions(),
    );
    let text = normalized_text(&buffer);
    for required in ["Carried loot: 2", "Extraction: Ready"] {
        assert!(
            text.contains(required),
            "contract=VISUAL-DUNGEON-001 fixture=exit-with-two-loot \
             missing dungeon status `{required}`\n{}",
            buffer_text(&buffer)
        );
    }
}

#[test]
fn title_without_save_explains_why_load_is_unavailable() {
    // Contract: VISUAL-SHELL-001
    for (width, height) in [(80, 24), (60, 20)] {
        let buffer = render_buffer(
            "title",
            width,
            height,
            GameMode::Title,
            &MapViewModel::default(),
            &StatsViewModel::default(),
            &LogViewModel::default(),
            &default_actions(),
        );
        let text = normalized_text(&buffer);
        assert!(
            text.contains("Load unavailable") && text.contains("No save"),
            "contract=VISUAL-SHELL-001 case={width}x{height} fixture=title-no-save \
             title advertises Load without explaining its unavailable state\n{}",
            buffer_text(&buffer)
        );
    }
}

#[test]
fn closed_management_modal_leaves_the_same_canvas_as_a_clean_overview() {
    // Contract: VISUAL-RESIZE-001 (modal-close stale-cell portion).
    let width = 80;
    let height = 24;
    let map = shelter_map();
    let actions = default_actions();
    let modal_stats = StatsViewModel {
        management: Some(view_models::ManagementMenuVm {
            kind: view_models::ManagementMenuKind::TaskAssignment,
            survivors: vec!["Mara — Idle".into()],
            tasks: vec!["1. Gather Supplies".into()],
            selected_survivor: Some(0),
            selected_task: None,
            resources: "Sup 4".into(),
            forecast: "Next worker: none".into(),
        }),
        ..Default::default()
    };
    let clean_stats = StatsViewModel::default();
    let screens = default_screen_registry();
    let widgets = default_widget_registry();
    let symbols = SymbolRegistry::phase5_defaults();
    let theme = ThemeRegistry::phase5_defaults();
    let bindings = commands::CommandBindings::default();
    let container = ContainerViewModel::default();
    let event = EventViewModel::default();
    let help = HelpViewModel::default();
    let log = LogViewModel::default();
    let definition = screens.get("outpost").expect("outpost must exist");
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("test terminal must initialize");

    for stats in [&modal_stats, &clean_stats] {
        let data = UiFrameData {
            definition,
            widgets: &widgets,
            map: &map,
            stats,
            log: &log,
            actions: &actions,
            container: &container,
            event: &event,
            help: &help,
            symbols: &symbols,
            theme: &theme,
            bindings: &bindings,
            mode: GameMode::Outpost,
            interaction: frame_interaction(
                GameMode::Outpost,
                false,
                stats.management.as_ref().map(|menu| menu.kind),
            ),
            turn: 0,
            day: 0,
        };
        terminal
            .draw(|frame| render_ui_frame(frame, &data))
            .expect("frame must render");
    }
    let after_close = terminal.backend().buffer().clone();
    let clean = render_buffer(
        "outpost",
        width,
        height,
        GameMode::Outpost,
        &map,
        &clean_stats,
        &log,
        &actions,
    );

    assert_eq!(
        observe(&after_close),
        observe(&clean),
        "contract=VISUAL-RESIZE-001 fixture=management-close \
         closed modal left stale cells"
    );
}

#[test]
fn resize_round_trip_returns_to_the_original_canvas_and_styles() {
    // Contract: VISUAL-RESIZE-001
    let map = shelter_map();
    let stats = StatsViewModel::default();
    let log = LogViewModel::default();
    let actions = default_actions();
    let original = render_buffer(
        "outpost",
        80,
        24,
        GameMode::Outpost,
        &map,
        &stats,
        &log,
        &actions,
    );
    let compact = render_buffer(
        "outpost",
        60,
        20,
        GameMode::Outpost,
        &map,
        &stats,
        &log,
        &actions,
    );
    let returned = render_buffer(
        "outpost",
        80,
        24,
        GameMode::Outpost,
        &map,
        &stats,
        &log,
        &actions,
    );

    assert_eq!(compact.area, Rect::from(Size::new(60, 20)));
    assert_eq!(
        observe(&original),
        observe(&returned),
        "contract=VISUAL-RESIZE-001 fixture=80-60-80 \
         round-trip resize changed the restored frame"
    );
}
