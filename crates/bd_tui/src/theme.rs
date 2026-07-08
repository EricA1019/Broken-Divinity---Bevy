//! Theme registry — maps StyleToken to Ratatui style properties.
//!
//! All color and style knowledge lives here. Move to RON in Phase 8.

use bevy_ecs::prelude::Resource;
use ratatui::style::{Color, Style};

use crate::visual::StyleToken;

/// Maps a StyleToken to a Ratatui Style.
#[derive(Debug, Clone)]
pub struct ThemeDef {
    pub style_token: StyleToken,
    pub fg: Color,
    pub bg: Option<Color>,
    pub bold: bool,
}

/// Registry of theme definitions.
#[derive(Resource, Debug, Clone, Default)]
pub struct ThemeRegistry {
    themes: Vec<ThemeDef>,
}

impl ThemeRegistry {
    pub fn new(themes: Vec<ThemeDef>) -> Self {
        Self { themes }
    }

    /// Resolve a StyleToken to a Ratatui Style.
    pub fn resolve(&self, token: StyleToken) -> Style {
        self.themes
            .iter()
            .find(|t| t.style_token == token)
            .map(|t| {
                let mut s = Style::default().fg(t.fg);
                if let Some(bg) = t.bg {
                    s = s.bg(bg);
                }
                if t.bold {
                    s = s.add_modifier(ratatui::style::Modifier::BOLD);
                }
                s
            })
            .unwrap_or_default()
    }

    /// Validate that all expected style tokens have definitions.
    #[allow(dead_code)]
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        let all_tokens = [
            StyleToken::Default,
            StyleToken::Player,
            StyleToken::Enemy,
            StyleToken::Ally,
            StyleToken::Terrain,
            StyleToken::Wall,
            StyleToken::Item,
            StyleToken::Exit,
            StyleToken::Danger,
            StyleToken::Muted,
            StyleToken::Selection,
        ];
        for token in &all_tokens {
            if !self.themes.iter().any(|t| t.style_token == *token) {
                errors.push(format!("Missing theme definition for {token:?}"));
            }
        }
        errors
    }

    /// Phase 5 default theme — Rust fixtures.
    pub fn phase5_defaults() -> Self {
        Self::new(vec![
            ThemeDef {
                style_token: StyleToken::Default,
                fg: Color::White,
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::Player,
                fg: Color::Yellow,
                bg: None,
                bold: true,
            },
            ThemeDef {
                style_token: StyleToken::Enemy,
                fg: Color::Red,
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::Ally,
                fg: Color::Green,
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::Terrain,
                fg: Color::Gray,
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::Wall,
                fg: Color::DarkGray,
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::Item,
                fg: Color::Cyan,
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::Exit,
                fg: Color::Magenta,
                bg: None,
                bold: true,
            },
            ThemeDef {
                style_token: StyleToken::Danger,
                fg: Color::Red,
                bg: None,
                bold: true,
            },
            ThemeDef {
                style_token: StyleToken::Muted,
                fg: Color::DarkGray,
                bg: None,
                bold: false,
            },
            ThemeDef {
                style_token: StyleToken::Selection,
                fg: Color::Black,
                bg: Some(Color::Yellow),
                bold: true,
            },
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_style_def_fails_validation() {
        let reg = ThemeRegistry::new(vec![]);
        let errors = reg.validate();
        assert!(
            !errors.is_empty(),
            "Empty theme registry should fail validation"
        );
    }

    #[test]
    fn player_style_resolves_to_yellow() {
        let reg = ThemeRegistry::phase5_defaults();
        let style = reg.resolve(StyleToken::Player);
        assert_eq!(style.fg, Some(Color::Yellow));
    }
}
