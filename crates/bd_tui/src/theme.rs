use bevy_ecs::prelude::Resource;
use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};

use crate::visual::StyleToken;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeDef {
    pub style_token: StyleToken,
    pub fg: String,
    pub bg: Option<String>,
    pub bold: bool,
}

impl ThemeDef {
    pub fn resolve(&self) -> Style {
        let mut s = Style::default().fg(parse_color(&self.fg));
        if let Some(ref bg) = self.bg {
            s = s.bg(parse_color(bg));
        }
        if self.bold {
            s = s.add_modifier(ratatui::style::Modifier::BOLD);
        }
        s
    }
}

fn parse_color(name: &str) -> Color {
    let trimmed = name.trim().trim_start_matches('#');
    if let Some(hex) = trimmed.strip_prefix("0x").or_else(|| {
        (trimmed.len() == 6 && trimmed.chars().all(|c| c.is_ascii_hexdigit())).then_some(trimmed)
    }) {
        if let Ok(value) = u32::from_str_radix(hex, 16) {
            return Color::Rgb(
                ((value >> 16) & 0xff) as u8,
                ((value >> 8) & 0xff) as u8,
                (value & 0xff) as u8,
            );
        }
    }
    match name.to_lowercase().as_str() {
        "white" => Color::White,
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" | "grey" => Color::Gray,
        "darkgray" | "dark_gray" | "darkgrey" | "dark_grey" => Color::DarkGray,
        "lightred" | "light_red" => Color::LightRed,
        "lightgreen" | "light_green" => Color::LightGreen,
        "lightyellow" | "light_yellow" => Color::LightYellow,
        "lightblue" | "light_blue" => Color::LightBlue,
        "lightmagenta" | "light_magenta" => Color::LightMagenta,
        "lightcyan" | "light_cyan" => Color::LightCyan,
        "reset" => Color::Reset,
        _ => Color::White,
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ThemeRegistry {
    pub themes: Vec<ThemeDef>,
}

impl ThemeRegistry {
    pub fn new(themes: Vec<ThemeDef>) -> Self {
        Self { themes }
    }

    pub fn from_defs(defs: Vec<ThemeDef>) -> Self {
        Self { themes: defs }
    }

    pub fn resolve(&self, token: StyleToken) -> Style {
        self.themes
            .iter()
            .find(|t| t.style_token == token)
            .map(|t| t.resolve())
            .unwrap_or_default()
    }

    pub fn validate(&self) -> Vec<String> {
        use std::collections::HashSet;

        let mut errors = Vec::new();
        let mut seen = HashSet::new();
        for theme in &self.themes {
            if !seen.insert(theme.style_token) {
                errors.push(format!(
                    "Duplicate theme definition for {:?}",
                    theme.style_token
                ));
            }
        }
        for token in &[
            StyleToken::Default,
            StyleToken::Player,
            StyleToken::Enemy,
            StyleToken::Ally,
            StyleToken::Terrain,
            StyleToken::Wall,
            StyleToken::Item,
            StyleToken::Station,
            StyleToken::ResourceNode,
            StyleToken::Exit,
            StyleToken::Danger,
            StyleToken::Muted,
            StyleToken::Selection,
            StyleToken::UiText,
            StyleToken::UiMuted,
            StyleToken::UiAccent,
            StyleToken::UiPositive,
            StyleToken::UiWarning,
            StyleToken::UiInfo,
            StyleToken::UiDanger,
            StyleToken::UiPanelBorder,
            StyleToken::UiPanelTitle,
            StyleToken::UiModalBorder,
            StyleToken::UiModalTitle,
            StyleToken::UiKeyHint,
            StyleToken::TitleWordmark,
        ] {
            if !self.themes.iter().any(|t| t.style_token == *token) {
                errors.push(format!("Missing theme definition for {token:?}"));
            }
        }
        errors
    }

    pub fn phase5_defaults() -> Self {
        Self::from_defs(vec![
            ThemeDef {
                style_token: StyleToken::Default,
                fg: "white".into(),
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::Player,
                fg: "yellow".into(),
                bg: None,
                bold: true,
            },
            ThemeDef {
                style_token: StyleToken::Enemy,
                fg: "red".into(),
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::Ally,
                fg: "green".into(),
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::Terrain,
                fg: "#dcc7b3".into(),
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::Wall,
                fg: "darkgray".into(),
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::Item,
                fg: "cyan".into(),
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::Station,
                fg: "lightblue".into(),
                bg: None,
                bold: true,
            },
            ThemeDef {
                style_token: StyleToken::ResourceNode,
                fg: "green".into(),
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::Exit,
                fg: "magenta".into(),
                bg: None,
                bold: true,
            },
            ThemeDef {
                style_token: StyleToken::Danger,
                fg: "red".into(),
                bg: None,
                bold: true,
            },
            ThemeDef {
                style_token: StyleToken::Muted,
                fg: "darkgray".into(),
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::Selection,
                fg: "#ffe1c6".into(),
                bg: Some("#5b2e20".into()),
                bold: true,
            },
            ThemeDef {
                style_token: StyleToken::UiText,
                fg: "#dcc7b3".into(),
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::UiMuted,
                fg: "#92786b".into(),
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::UiAccent,
                fg: "#b76b4c".into(),
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::UiPositive,
                fg: "#8d9d62".into(),
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::UiWarning,
                fg: "#e0a13f".into(),
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::UiInfo,
                fg: "#a68ab0".into(),
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::UiDanger,
                fg: "#d15348".into(),
                bg: None,
                bold: true,
            },
            ThemeDef {
                style_token: StyleToken::UiPanelBorder,
                fg: "#714737".into(),
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::UiPanelTitle,
                fg: "#dd8a50".into(),
                bg: None,
                bold: true,
            },
            ThemeDef {
                style_token: StyleToken::UiModalBorder,
                fg: "#b76b4c".into(),
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::UiModalTitle,
                fg: "#dd8a50".into(),
                bg: None,
                bold: true,
            },
            ThemeDef {
                style_token: StyleToken::UiKeyHint,
                fg: "#a68ab0".into(),
                bg: None,
                bold: true,
            },
            ThemeDef {
                style_token: StyleToken::TitleWordmark,
                fg: "cyan".into(),
                bg: None,
                bold: true,
            },
        ])
    }
}
