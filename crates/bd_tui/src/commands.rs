//! Semantic terminal commands and their projections into input and guidance.
//!
//! A configured binding is resolved here once, then reused by input routing,
//! help, the footer, and the action panel.

use bevy_ecs::prelude::Resource;
use crossterm::event::KeyCode;

use bd_core::spatial::GameMode;

pub const FULL_TERMINAL_WIDTH: u16 = 80;
pub const FULL_TERMINAL_HEIGHT: u16 = 24;
pub const MIN_TERMINAL_WIDTH: u16 = 60;
pub const MIN_TERMINAL_HEIGHT: u16 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiCommand {
    MoveNorth,
    MoveSouth,
    MoveEast,
    MoveWest,
    Wait,
    RestUntilNextDay,
    Attack,
    Guard,
    Inventory,
    Pickup,
    UseItem,
    Help,
    Travel,
    Extract,
    AssignTask,
    AssignStation,
    Build,
    Save,
    Load,
    Quit,
    Restart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionMode {
    Normal,
    Build,
    TaskManagement,
    StationStaffing,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalLayout {
    Full,
    Compact,
    TooSmall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleInput {
    Begin,
    Load,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOverInput {
    Restart,
    Save,
    Load,
    Quit,
}

pub fn title_input(bindings: &CommandBindings, key: &KeyCode) -> TitleInput {
    match bindings.command_for_key_in(key, GameMode::Title, InteractionMode::Normal) {
        Some(UiCommand::Load) => TitleInput::Load,
        Some(UiCommand::Quit) => TitleInput::Quit,
        _ => TitleInput::Begin,
    }
}

pub fn game_over_input(bindings: &CommandBindings, key: &KeyCode) -> Option<GameOverInput> {
    match bindings.command_for_key_in(key, GameMode::GameOver, InteractionMode::GameOver) {
        Some(UiCommand::Restart) => Some(GameOverInput::Restart),
        Some(UiCommand::Save) => Some(GameOverInput::Save),
        Some(UiCommand::Load) => Some(GameOverInput::Load),
        Some(UiCommand::Quit) => Some(GameOverInput::Quit),
        _ => None,
    }
}

pub fn inventory_toggle_destination(current_screen: &str, mode: GameMode) -> &'static str {
    if current_screen != "inventory" {
        "inventory"
    } else if mode == GameMode::Outpost {
        "outpost"
    } else {
        "combat"
    }
}

pub fn terminal_layout(width: u16, height: u16) -> TerminalLayout {
    if width < MIN_TERMINAL_WIDTH || height < MIN_TERMINAL_HEIGHT {
        TerminalLayout::TooSmall
    } else if width < FULL_TERMINAL_WIDTH || height < FULL_TERMINAL_HEIGHT {
        TerminalLayout::Compact
    } else {
        TerminalLayout::Full
    }
}

pub fn command_action_id(command: UiCommand) -> Option<&'static str> {
    match command {
        UiCommand::MoveNorth | UiCommand::MoveSouth | UiCommand::MoveEast | UiCommand::MoveWest => {
            Some("ability.move")
        }
        UiCommand::Wait => Some("ability.wait"),
        UiCommand::RestUntilNextDay => Some("ability.rest_until_next_day"),
        UiCommand::Attack => Some("ability.quick_attack"),
        UiCommand::Guard => Some("ability.guard"),
        UiCommand::Pickup => Some("ability.pickup"),
        UiCommand::UseItem => Some("ability.use_item"),
        UiCommand::Extract => Some("ability.extract"),
        UiCommand::AssignStation => Some("ability.assign_station"),
        UiCommand::Build => Some("ability.build"),
        _ => None,
    }
}

pub(crate) fn is_buffered_gameplay(command: UiCommand) -> bool {
    matches!(
        command,
        UiCommand::MoveNorth
            | UiCommand::MoveSouth
            | UiCommand::MoveEast
            | UiCommand::MoveWest
            | UiCommand::Wait
            | UiCommand::RestUntilNextDay
            | UiCommand::Attack
            | UiCommand::Guard
            | UiCommand::Pickup
            | UiCommand::UseItem
            | UiCommand::Travel
            | UiCommand::Extract
            | UiCommand::AssignTask
            | UiCommand::AssignStation
    )
}

#[derive(Resource, Debug, Clone)]
pub struct CommandBindings {
    bindings: Vec<(UiCommand, KeyCode)>,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ApplicationExitRequest(pub bool);

impl Default for CommandBindings {
    fn default() -> Self {
        Self {
            bindings: vec![
                (UiCommand::MoveNorth, KeyCode::Char('w')),
                (UiCommand::MoveSouth, KeyCode::Char('s')),
                (UiCommand::MoveEast, KeyCode::Char('d')),
                (UiCommand::MoveWest, KeyCode::Char('a')),
                (UiCommand::Wait, KeyCode::Char('.')),
                (UiCommand::RestUntilNextDay, KeyCode::Char('n')),
                (UiCommand::Attack, KeyCode::Char('f')),
                (UiCommand::Guard, KeyCode::Char('g')),
                (UiCommand::Inventory, KeyCode::Char('i')),
                (UiCommand::Pickup, KeyCode::Char('p')),
                (UiCommand::UseItem, KeyCode::Char('u')),
                (UiCommand::Help, KeyCode::Char('?')),
                (UiCommand::Travel, KeyCode::Char('t')),
                (UiCommand::Extract, KeyCode::Char('r')),
                (UiCommand::AssignTask, KeyCode::Char('c')),
                (UiCommand::AssignStation, KeyCode::Char('e')),
                (UiCommand::Build, KeyCode::Char('b')),
                (UiCommand::Save, KeyCode::F(5)),
                (UiCommand::Load, KeyCode::F(9)),
                (UiCommand::Quit, KeyCode::Char('q')),
                (UiCommand::Restart, KeyCode::Char('r')),
            ],
        }
    }
}

impl CommandBindings {
    pub fn bind(&mut self, command: UiCommand, key: KeyCode) {
        if let Some((_, current)) = self
            .bindings
            .iter_mut()
            .find(|(candidate, _)| *candidate == command)
        {
            *current = key;
        } else {
            self.bindings.push((command, key));
        }
    }

    pub fn key_for(&self, command: UiCommand) -> Option<&KeyCode> {
        self.bindings
            .iter()
            .find_map(|(candidate, key)| (*candidate == command).then_some(key))
    }

    pub fn command_for_key_in(
        &self,
        key: &KeyCode,
        mode: GameMode,
        interaction: InteractionMode,
    ) -> Option<UiCommand> {
        command_order(mode, interaction)
            .iter()
            .copied()
            .find(|command| self.key_for(*command).is_some_and(|bound| bound == key))
            .or_else(|| arrow_alias(key, interaction))
            .or_else(|| (*key == KeyCode::Esc).then_some(UiCommand::Quit))
    }

    /// Resolve a key without a gameplay context. This is intended for config
    /// validation and tests; runtime routing uses [`Self::command_for_key_in`].
    pub fn command_for_key(&self, key: &KeyCode) -> Option<UiCommand> {
        self.bindings
            .iter()
            .find_map(|(command, bound)| (bound == key).then_some(*command))
    }

    pub fn conflicts(&self) -> Vec<(UiCommand, UiCommand, String)> {
        let contexts = [
            (GameMode::Outpost, InteractionMode::Normal),
            (GameMode::Tactical, InteractionMode::Normal),
            (GameMode::Outpost, InteractionMode::Build),
            (GameMode::GameOver, InteractionMode::GameOver),
        ];
        let mut conflicts = Vec::new();
        for (mode, interaction) in contexts {
            let commands = command_order(mode, interaction);
            for (index, left) in commands.iter().copied().enumerate() {
                let Some(left_key) = self.key_for(left) else {
                    continue;
                };
                for right in commands.iter().copied().skip(index + 1) {
                    if self.key_for(right).is_some_and(|key| key == left_key)
                        && !conflicts.iter().any(|(existing_left, existing_right, _)| {
                            (*existing_left == left && *existing_right == right)
                                || (*existing_left == right && *existing_right == left)
                        })
                    {
                        conflicts.push((left, right, key_label(left_key)));
                    }
                }
            }
        }
        conflicts
    }
}

fn arrow_alias(key: &KeyCode, interaction: InteractionMode) -> Option<UiCommand> {
    match (key, interaction) {
        (KeyCode::Up, _) => Some(UiCommand::MoveNorth),
        (KeyCode::Down, _) => Some(UiCommand::MoveSouth),
        (KeyCode::Right, _) => Some(UiCommand::MoveEast),
        (KeyCode::Left, _) => Some(UiCommand::MoveWest),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpEntry {
    pub command: UiCommand,
    pub key: String,
    pub label: String,
    pub description: String,
    pub kind: HelpEntryKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpEntryKind {
    Control,
    Legend,
}

#[derive(Debug, Clone, Copy)]
struct Descriptor {
    command: UiCommand,
    label: &'static str,
    description: &'static str,
}

fn descriptor(command: UiCommand) -> Descriptor {
    let (label, description) = match command {
        UiCommand::MoveNorth => ("Move", "Move one tile"),
        UiCommand::MoveSouth => ("Move south", "Move one tile south"),
        UiCommand::MoveEast => ("Move east", "Move one tile east"),
        UiCommand::MoveWest => ("Move west", "Move one tile west"),
        UiCommand::Wait => ("Wait", "End the turn"),
        UiCommand::RestUntilNextDay => ("Rest", "Rest to next day"),
        UiCommand::Attack => ("Attack", "Attack nearest enemy"),
        UiCommand::Guard => ("Guard", "Guard against damage"),
        UiCommand::Inventory => ("Inventory", "Open inventory"),
        UiCommand::Pickup => ("Pickup", "Pick up item"),
        UiCommand::UseItem => ("Use", "Use carried item"),
        UiCommand::Help => ("Help", "Show or close Help"),
        UiCommand::Travel => ("Travel", "Enter dungeon"),
        UiCommand::Extract => ("Extract", "Leave dungeon"),
        UiCommand::AssignTask => ("Assign task", "Manage survivor tasks"),
        UiCommand::AssignStation => ("Staff station", "Staff stations"),
        UiCommand::Build => ("Build", "Build a station"),
        UiCommand::Save => ("Save", "Save game"),
        UiCommand::Load => ("Load", "Load game"),
        UiCommand::Quit => ("Quit", "Quit"),
        UiCommand::Restart => ("Restart", "Return to title"),
    };
    Descriptor {
        command,
        label,
        description,
    }
}

fn command_order(mode: GameMode, interaction: InteractionMode) -> &'static [UiCommand] {
    const BUILD: &[UiCommand] = &[
        UiCommand::MoveNorth,
        UiCommand::MoveSouth,
        UiCommand::MoveEast,
        UiCommand::MoveWest,
        UiCommand::Build,
        UiCommand::Quit,
    ];
    const GAME_OVER: &[UiCommand] = &[
        UiCommand::Restart,
        UiCommand::Save,
        UiCommand::Load,
        UiCommand::Quit,
    ];
    const MANAGEMENT: &[UiCommand] = &[];
    const TITLE: &[UiCommand] = &[UiCommand::Load, UiCommand::Quit];
    const COLONY: &[UiCommand] = &[
        UiCommand::Travel,
        UiCommand::MoveNorth,
        UiCommand::MoveSouth,
        UiCommand::MoveEast,
        UiCommand::MoveWest,
        UiCommand::RestUntilNextDay,
        UiCommand::Build,
        UiCommand::Inventory,
        UiCommand::UseItem,
        UiCommand::Wait,
        UiCommand::AssignTask,
        UiCommand::AssignStation,
        UiCommand::Help,
        UiCommand::Save,
        UiCommand::Load,
        UiCommand::Quit,
    ];
    const DUNGEON: &[UiCommand] = &[
        UiCommand::Extract,
        UiCommand::Attack,
        UiCommand::Inventory,
        UiCommand::UseItem,
        UiCommand::MoveNorth,
        UiCommand::MoveSouth,
        UiCommand::MoveEast,
        UiCommand::MoveWest,
        UiCommand::Wait,
        UiCommand::Guard,
        UiCommand::Pickup,
        UiCommand::Help,
        UiCommand::Save,
        UiCommand::Load,
        UiCommand::Quit,
    ];
    const GLOBAL: &[UiCommand] = &[
        UiCommand::Help,
        UiCommand::Save,
        UiCommand::Load,
        UiCommand::Quit,
    ];

    match interaction {
        InteractionMode::Build => BUILD,
        InteractionMode::TaskManagement | InteractionMode::StationStaffing => MANAGEMENT,
        InteractionMode::GameOver => GAME_OVER,
        InteractionMode::Normal => match mode {
            GameMode::Title => TITLE,
            GameMode::Outpost => COLONY,
            GameMode::Tactical => DUNGEON,
            _ => GLOBAL,
        },
    }
}

fn key_label(key: &KeyCode) -> String {
    match key {
        KeyCode::Char(value) => value.to_string(),
        KeyCode::F(number) => format!("F{number}"),
        KeyCode::Esc => "Esc".into(),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Up => "↑".into(),
        KeyCode::Down => "↓".into(),
        KeyCode::Left => "←".into(),
        KeyCode::Right => "→".into(),
        other => format!("{other:?}"),
    }
}

/// Serialize a supported binding into the user-configuration spelling.
pub fn config_key_name(key: &KeyCode) -> String {
    match key {
        KeyCode::Char(value) => value.to_string(),
        KeyCode::F(number) => format!("F{number}"),
        KeyCode::Esc => "Esc".into(),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Up => "Up".into(),
        KeyCode::Down => "Down".into(),
        KeyCode::Left => "Left".into(),
        KeyCode::Right => "Right".into(),
        other => format!("{other:?}"),
    }
}

fn display_key(bindings: &CommandBindings, command: UiCommand) -> String {
    if command == UiCommand::MoveNorth {
        let keys = [
            UiCommand::MoveNorth,
            UiCommand::MoveWest,
            UiCommand::MoveSouth,
            UiCommand::MoveEast,
        ]
        .into_iter()
        .filter_map(|direction| bindings.key_for(direction))
        .map(key_label)
        .collect::<Vec<_>>()
        .join("");
        return format!("{keys}/arrows");
    }
    bindings
        .key_for(command)
        .map(key_label)
        .unwrap_or_else(|| "unbound".into())
}

pub fn help_entries(
    bindings: &CommandBindings,
    mode: GameMode,
    interaction: InteractionMode,
) -> Vec<HelpEntry> {
    command_order(mode, interaction)
        .iter()
        .copied()
        .filter(|command| {
            !matches!(
                command,
                UiCommand::MoveSouth | UiCommand::MoveEast | UiCommand::MoveWest
            )
        })
        .map(|command| {
            let descriptor = descriptor(command);
            HelpEntry {
                command: descriptor.command,
                key: display_key(bindings, command),
                label: descriptor.label.into(),
                description: descriptor.description.into(),
                kind: HelpEntryKind::Control,
            }
        })
        .collect()
}

pub fn help_entries_with_legend(
    bindings: &CommandBindings,
    mode: GameMode,
    interaction: InteractionMode,
    symbols: &crate::visual::SymbolRegistry,
    stations: &bd_core::colony::stations::StationCatalog,
) -> Vec<HelpEntry> {
    use crate::visual::VisualToken;

    fn glyph(symbols: &crate::visual::SymbolRegistry, token: VisualToken) -> String {
        symbols
            .get(token)
            .map_or_else(|| "?".into(), |symbol| symbol.glyph.to_string())
    }

    let mut entries = help_entries(bindings, mode, interaction);
    if interaction != InteractionMode::Normal {
        return entries;
    }

    let mut legend = Vec::new();
    let mut add = |key: String, label: &str, description: &str| {
        legend.push(HelpEntry {
            command: UiCommand::Help,
            key,
            label: label.into(),
            description: description.into(),
            kind: HelpEntryKind::Legend,
        });
    };
    add(glyph(symbols, VisualToken::Player), "Player", "Player");
    if mode == GameMode::Outpost {
        add(
            glyph(symbols, VisualToken::Trees),
            "Trees",
            "Trees: Materials",
        );
        add(
            glyph(symbols, VisualToken::WaterSource),
            "Water Source",
            "Water Source: Supplies",
        );
        add(
            glyph(symbols, VisualToken::WildPlants),
            "Wild Plants",
            "Wild Plants: Medicine",
        );
        let worker_glyphs = [
            VisualToken::WorkerIdle,
            VisualToken::WorkerEnRoute,
            VisualToken::WorkerWorking,
            VisualToken::WorkerBlocked,
            VisualToken::WorkerResting,
            VisualToken::WorkerDefending,
        ]
        .map(|token| glyph(symbols, token))
        .join("/");
        add(worker_glyphs, "Workers", "Worker states");
        const STATIONS_PER_HELP_ENTRY: usize = 3;
        for station_group in stations.entries().chunks(STATIONS_PER_HELP_ENTRY) {
            let station_glyphs = station_group
                .iter()
                .map(|station| format!("{}/{}", station.glyph, station.staffed_glyph))
                .collect::<Vec<_>>()
                .join(" ");
            add(station_glyphs, "Stations", "Stations");
        }
        add(
            glyph(symbols, VisualToken::Exit),
            "Shelter gate",
            "Shelter gate",
        );
        add("↑↓←→".into(), "Off-screen target", "Off-screen target");
    } else if mode == GameMode::Tactical {
        add(
            format!("{}/r/S/B", glyph(symbols, VisualToken::Enemy)),
            "Enemy",
            "Enemy",
        );
        add(glyph(symbols, VisualToken::Item), "Item", "Loot item");
        add(glyph(symbols, VisualToken::Exit), "Exit", "Dungeon exit");
    }
    entries.extend(legend);
    entries
}

pub fn footer_text(
    bindings: &CommandBindings,
    mode: GameMode,
    interaction: InteractionMode,
) -> String {
    help_entries(bindings, mode, interaction)
        .into_iter()
        .filter(|entry| entry.kind == HelpEntryKind::Control)
        .map(|entry| format!("{}:{}", entry.label, entry.key))
        .collect::<Vec<_>>()
        .join(" | ")
}

#[derive(Debug, Clone, Copy)]
pub struct ActionAvailability {
    pub mode: GameMode,
    pub has_ap: bool,
    pub enemy_in_range: bool,
    pub can_move: bool,
    pub item_here: bool,
    pub usable_item: bool,
    pub can_build: bool,
    pub survivor_available: bool,
    pub station_available: bool,
    pub at_exit: bool,
    pub can_travel: bool,
    pub day: u64,
    pub turn: u64,
}

impl ActionAvailability {
    pub fn dungeon(
        has_ap: bool,
        enemy_in_range: bool,
        can_move: bool,
        item_here: bool,
        usable_item: bool,
    ) -> Self {
        Self {
            mode: GameMode::Tactical,
            has_ap,
            enemy_in_range,
            can_move,
            item_here,
            usable_item,
            can_build: false,
            survivor_available: false,
            station_available: false,
            at_exit: false,
            can_travel: false,
            day: 0,
            turn: 0,
        }
    }

    pub fn outpost(
        has_ap: bool,
        can_move: bool,
        can_build: bool,
        survivor_available: bool,
        station_available: bool,
    ) -> Self {
        Self {
            mode: GameMode::Outpost,
            has_ap,
            enemy_in_range: false,
            can_move,
            item_here: false,
            usable_item: false,
            can_build,
            survivor_available,
            station_available,
            at_exit: false,
            can_travel: true,
            day: 0,
            turn: 0,
        }
    }

    pub fn at_exit(mut self, at_exit: bool) -> Self {
        self.at_exit = at_exit;
        self
    }

    pub fn can_travel(mut self, can_travel: bool) -> Self {
        self.can_travel = can_travel;
        self
    }

    pub fn time(mut self, day: u64, turn: u64) -> Self {
        self.day = day;
        self.turn = turn;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionProjection {
    pub command: UiCommand,
    pub label: String,
    pub key: String,
    pub enabled: bool,
    pub denial_reason: Option<String>,
}

pub fn action_panel(
    bindings: &CommandBindings,
    availability: ActionAvailability,
) -> Vec<ActionProjection> {
    let commands: &[UiCommand] = match availability.mode {
        GameMode::Outpost => &[
            UiCommand::Travel,
            UiCommand::Build,
            UiCommand::AssignTask,
            UiCommand::AssignStation,
            UiCommand::RestUntilNextDay,
            UiCommand::MoveNorth,
            UiCommand::Wait,
        ],
        GameMode::Tactical => &[
            UiCommand::Extract,
            UiCommand::Attack,
            UiCommand::MoveNorth,
            UiCommand::Wait,
            UiCommand::Guard,
            UiCommand::Pickup,
            UiCommand::UseItem,
        ],
        _ => &[],
    };

    commands
        .iter()
        .copied()
        .filter(|command| *command != UiCommand::Extract || availability.at_exit)
        .map(|command| {
            let (enabled, denial_reason) = match command {
                UiCommand::MoveNorth => {
                    if !availability.has_ap {
                        (false, Some("No AP".into()))
                    } else if !availability.can_move {
                        (false, Some("Blocked".into()))
                    } else {
                        (true, None)
                    }
                }
                UiCommand::Attack => {
                    if !availability.has_ap {
                        (false, Some("No AP".into()))
                    } else if !availability.enemy_in_range {
                        (false, Some("No target in range".into()))
                    } else {
                        (true, None)
                    }
                }
                UiCommand::Guard => {
                    if availability.has_ap {
                        (true, None)
                    } else {
                        (false, Some("No AP".into()))
                    }
                }
                UiCommand::Pickup => {
                    if availability.item_here {
                        (true, None)
                    } else {
                        (false, Some("Nothing here".into()))
                    }
                }
                UiCommand::UseItem => {
                    if availability.usable_item {
                        (true, None)
                    } else {
                        (false, Some("No usable item".into()))
                    }
                }
                UiCommand::Build => {
                    if availability.can_build {
                        (true, None)
                    } else {
                        (false, Some("Insufficient supplies".into()))
                    }
                }
                UiCommand::AssignTask => {
                    if availability.survivor_available {
                        (true, None)
                    } else {
                        (false, Some("No survivor".into()))
                    }
                }
                UiCommand::AssignStation => {
                    if !availability.survivor_available {
                        (false, Some("No survivor".into()))
                    } else if !availability.station_available {
                        (false, Some("No station".into()))
                    } else {
                        (true, None)
                    }
                }
                UiCommand::Travel => {
                    if availability.can_travel {
                        (true, None)
                    } else {
                        (
                            false,
                            Some(format!(
                                "Need {} Supplies",
                                bd_core::spatial::TRAVEL_SUPPLIES_COST
                            )),
                        )
                    }
                }
                _ => (true, None),
            };
            let descriptor = descriptor(command);
            let label = if command == UiCommand::RestUntilNextDay {
                format!(
                    "Rest to Day {} ({} turns)",
                    availability.day + 1,
                    bd_core::time::TURNS_PER_DAY - availability.turn
                )
            } else {
                descriptor.label.into()
            };
            ActionProjection {
                command,
                label,
                key: display_key(bindings, command),
                enabled,
                denial_reason,
            }
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FooterControlLines {
    pub contextual: String,
    pub global: String,
}

pub fn footer_control_lines(
    bindings: &CommandBindings,
    mode: GameMode,
    interaction: InteractionMode,
    screen_id: &str,
    width: u16,
) -> FooterControlLines {
    fn pack(tokens: &[String], width: usize) -> String {
        let mut line = String::new();
        for token in tokens {
            let separator = if line.is_empty() { "" } else { " | " };
            if line.len() + separator.len() + token.len() > width {
                break;
            }
            line.push_str(separator);
            line.push_str(token);
        }
        line
    }

    if matches!(
        interaction,
        InteractionMode::TaskManagement | InteractionMode::StationStaffing
    ) {
        let cancel = match interaction {
            InteractionMode::TaskManagement => "c/Esc:cancel",
            InteractionMode::StationStaffing => "e/Esc:cancel",
            _ => unreachable!("management footer branch requires a management interaction"),
        };
        return FooterControlLines {
            contextual: pack(
                &["1-9:select".into(), "Enter:confirm".into(), cancel.into()],
                width as usize,
            ),
            global: String::new(),
        };
    }

    let inventory = screen_id == "inventory";
    let mut contextual = Vec::new();
    let mut global = Vec::new();
    for entry in help_entries(bindings, mode, interaction) {
        if entry.kind != HelpEntryKind::Control {
            continue;
        }
        if entry.command == UiCommand::Extract {
            continue;
        }
        let label = if inventory && entry.command == UiCommand::Inventory {
            "Back"
        } else if interaction == InteractionMode::Build && entry.command == UiCommand::Quit {
            "Cancel"
        } else {
            entry.label.as_str()
        };
        let token = format!("{label}:{}", entry.key);
        let is_global = matches!(
            entry.command,
            UiCommand::Help
                | UiCommand::Save
                | UiCommand::Load
                | UiCommand::Quit
                | UiCommand::Restart
        ) || (inventory
            && matches!(entry.command, UiCommand::Inventory | UiCommand::UseItem));
        if is_global {
            global.push(token);
        } else {
            contextual.push(token);
        }
    }

    FooterControlLines {
        contextual: pack(&contextual, width as usize),
        global: pack(&global, width as usize),
    }
}
