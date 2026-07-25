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
    Attack,
    Guard,
    Inventory,
    CombatScreen,
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
                (UiCommand::Attack, KeyCode::Char('f')),
                (UiCommand::Guard, KeyCode::Char('g')),
                (UiCommand::Inventory, KeyCode::Char('i')),
                (UiCommand::CombatScreen, KeyCode::Char('z')),
                (UiCommand::Pickup, KeyCode::Char('p')),
                (UiCommand::UseItem, KeyCode::Char('u')),
                (UiCommand::Help, KeyCode::Char('?')),
                (UiCommand::Travel, KeyCode::Char('t')),
                (UiCommand::Extract, KeyCode::Char('r')),
                (UiCommand::AssignTask, KeyCode::Char('a')),
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
    pub label: &'static str,
    pub description: &'static str,
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
        UiCommand::Attack => ("Attack", "Attack nearest enemy"),
        UiCommand::Guard => ("Guard", "Guard against damage"),
        UiCommand::Inventory => ("Inventory", "Open inventory"),
        UiCommand::CombatScreen => ("Map", "Return to the map"),
        UiCommand::Pickup => ("Pickup", "Pick up item"),
        UiCommand::UseItem => ("Use", "Use carried item"),
        UiCommand::Help => ("Help", "Toggle contextual help"),
        UiCommand::Travel => ("Travel", "Enter the foundation dungeon"),
        UiCommand::Extract => ("Extract", "Leave the dungeon"),
        UiCommand::AssignTask => ("Assign task", "Cycle nearest survivor task"),
        UiCommand::AssignStation => ("Staff station", "Assign survivor to station"),
        UiCommand::Build => ("Build", "Build or cancel station"),
        UiCommand::Save => ("Save", "Save the current run"),
        UiCommand::Load => ("Load", "Load the manual save"),
        UiCommand::Quit => ("Quit", "Quit Broken Divinity"),
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
    const TITLE: &[UiCommand] = &[UiCommand::Load, UiCommand::Quit];
    const COLONY: &[UiCommand] = &[
        UiCommand::MoveNorth,
        UiCommand::MoveSouth,
        UiCommand::MoveEast,
        UiCommand::MoveWest,
        UiCommand::Wait,
        UiCommand::Inventory,
        UiCommand::UseItem,
        UiCommand::Build,
        UiCommand::AssignTask,
        UiCommand::AssignStation,
        UiCommand::Travel,
        UiCommand::Help,
        UiCommand::Save,
        UiCommand::Load,
        UiCommand::Quit,
    ];
    const DUNGEON: &[UiCommand] = &[
        UiCommand::MoveNorth,
        UiCommand::MoveSouth,
        UiCommand::MoveEast,
        UiCommand::MoveWest,
        UiCommand::Wait,
        UiCommand::Attack,
        UiCommand::Guard,
        UiCommand::Inventory,
        UiCommand::CombatScreen,
        UiCommand::Pickup,
        UiCommand::UseItem,
        UiCommand::Extract,
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
                label: descriptor.label,
                description: descriptor.description,
            }
        })
        .collect()
}

pub fn footer_text(
    bindings: &CommandBindings,
    mode: GameMode,
    interaction: InteractionMode,
) -> String {
    help_entries(bindings, mode, interaction)
        .into_iter()
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionProjection {
    pub command: UiCommand,
    pub label: &'static str,
    pub key: String,
    pub enabled: bool,
    pub denial_reason: Option<&'static str>,
}

pub fn action_panel(
    bindings: &CommandBindings,
    availability: ActionAvailability,
) -> Vec<ActionProjection> {
    let commands: &[UiCommand] = match availability.mode {
        GameMode::Outpost => &[
            UiCommand::MoveNorth,
            UiCommand::Wait,
            UiCommand::Build,
            UiCommand::AssignTask,
            UiCommand::AssignStation,
            UiCommand::Travel,
        ],
        GameMode::Tactical => &[
            UiCommand::MoveNorth,
            UiCommand::Wait,
            UiCommand::Attack,
            UiCommand::Guard,
            UiCommand::Pickup,
            UiCommand::UseItem,
            UiCommand::Extract,
        ],
        _ => &[],
    };

    commands
        .iter()
        .copied()
        .map(|command| {
            let (enabled, denial_reason) = match command {
                UiCommand::MoveNorth => {
                    if !availability.has_ap {
                        (false, Some("No AP"))
                    } else if !availability.can_move {
                        (false, Some("Blocked"))
                    } else {
                        (true, None)
                    }
                }
                UiCommand::Attack => {
                    if !availability.has_ap {
                        (false, Some("No AP"))
                    } else if !availability.enemy_in_range {
                        (false, Some("No target in range"))
                    } else {
                        (true, None)
                    }
                }
                UiCommand::Guard => {
                    if availability.has_ap {
                        (true, None)
                    } else {
                        (false, Some("No AP"))
                    }
                }
                UiCommand::Pickup => {
                    if availability.item_here {
                        (true, None)
                    } else {
                        (false, Some("Nothing here"))
                    }
                }
                UiCommand::UseItem => {
                    if availability.usable_item {
                        (true, None)
                    } else {
                        (false, Some("No usable item"))
                    }
                }
                UiCommand::Build => {
                    if availability.can_build {
                        (true, None)
                    } else {
                        (false, Some("Insufficient supplies"))
                    }
                }
                UiCommand::AssignTask => {
                    if availability.survivor_available {
                        (true, None)
                    } else {
                        (false, Some("No survivor"))
                    }
                }
                UiCommand::AssignStation => {
                    if !availability.survivor_available {
                        (false, Some("No survivor"))
                    } else if !availability.station_available {
                        (false, Some("No station"))
                    } else {
                        (true, None)
                    }
                }
                _ => (true, None),
            };
            let descriptor = descriptor(command);
            ActionProjection {
                command,
                label: descriptor.label,
                key: display_key(bindings, command),
                enabled,
                denial_reason,
            }
        })
        .collect()
}
