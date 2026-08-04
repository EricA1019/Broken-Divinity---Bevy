//! Visual tokens — semantic rendering symbols, decoupled from gameplay.
//!
//! Gameplay code emits `VisualToken`, never raw glyphs. The renderer resolves
//! tokens to glyphs and styles through the SymbolRegistry.

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

/// What kind of thing is being rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VisualToken {
    Player,
    Enemy,
    Ally,
    WorkerIdle,
    WorkerEnRoute,
    WorkerWorking,
    WorkerBlocked,
    WorkerResting,
    WorkerDefending,
    Floor,
    Wall,
    DoorClosed,
    DoorOpen,
    Item,
    Station,
    ResourceNode,
    Trees,
    WaterSource,
    WildPlants,
    TargetIndicator,
    Exit,
    Selection,
    InvalidSelection,
    Fog,
    Water,
}

/// What visual style to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StyleToken {
    Default,
    Player,
    Enemy,
    Ally,
    Terrain,
    Wall,
    Item,
    Station,
    ResourceNode,
    Exit,
    Danger,
    Muted,
    Selection,
    UiText,
    UiMuted,
    UiAccent,
    UiPositive,
    UiWarning,
    UiInfo,
    UiDanger,
    UiPanelBorder,
    UiPanelTitle,
    UiModalBorder,
    UiModalTitle,
    UiKeyHint,
    /// Brand wordmark on the title screen — deliberately distinct from the
    /// Cinder Rite panel/modal title copper so the product mark stays stable.
    TitleWordmark,
}

/// Maps a VisualToken to its glyph and style.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolDef {
    pub visual_token: VisualToken,
    pub glyph: char,
    #[allow(dead_code)]
    pub fallback_glyph: char,
    pub layer: u8,
    pub style_token: StyleToken,
    pub priority: u8,
}

/// Registry of all symbol definitions.
#[derive(Resource, Debug, Clone, Default)]
pub struct SymbolRegistry {
    symbols: Vec<SymbolDef>,
}

impl SymbolRegistry {
    pub fn new(symbols: Vec<SymbolDef>) -> Self {
        Self { symbols }
    }

    /// Look up a symbol by token.
    pub fn get(&self, token: VisualToken) -> Option<&SymbolDef> {
        self.symbols.iter().find(|s| s.visual_token == token)
    }

    /// Validate that all expected tokens have definitions.
    #[allow(dead_code)]
    pub fn validate(&self) -> Vec<String> {
        use std::collections::HashSet;

        let mut errors = Vec::new();
        let mut seen = HashSet::new();
        for symbol in &self.symbols {
            if !seen.insert(symbol.visual_token) {
                errors.push(format!(
                    "Duplicate symbol definition for {:?}",
                    symbol.visual_token
                ));
            }
        }
        let all_tokens = [
            VisualToken::Player,
            VisualToken::Enemy,
            VisualToken::Ally,
            VisualToken::WorkerIdle,
            VisualToken::WorkerEnRoute,
            VisualToken::WorkerWorking,
            VisualToken::WorkerBlocked,
            VisualToken::WorkerResting,
            VisualToken::WorkerDefending,
            VisualToken::Floor,
            VisualToken::Wall,
            VisualToken::DoorClosed,
            VisualToken::DoorOpen,
            VisualToken::Item,
            VisualToken::Station,
            VisualToken::ResourceNode,
            VisualToken::Trees,
            VisualToken::WaterSource,
            VisualToken::WildPlants,
            VisualToken::TargetIndicator,
            VisualToken::Exit,
            VisualToken::Selection,
            VisualToken::InvalidSelection,
            VisualToken::Fog,
        ];
        for token in &all_tokens {
            if self.get(*token).is_none() {
                errors.push(format!("Missing symbol definition for {token:?}"));
            }
        }
        errors
    }

    /// Phase 5 default symbols — Rust fixtures, move to RON in Phase 8.
    pub fn phase5_defaults() -> Self {
        Self::new(vec![
            SymbolDef {
                visual_token: VisualToken::Player,
                glyph: '@',
                fallback_glyph: '?',
                layer: 10,
                style_token: StyleToken::Player,
                priority: 10,
            },
            SymbolDef {
                visual_token: VisualToken::Enemy,
                glyph: 'E',
                fallback_glyph: '?',
                layer: 9,
                style_token: StyleToken::Enemy,
                priority: 9,
            },
            SymbolDef {
                visual_token: VisualToken::Ally,
                glyph: 'A',
                fallback_glyph: '?',
                layer: 9,
                style_token: StyleToken::Ally,
                priority: 9,
            },
            SymbolDef {
                visual_token: VisualToken::WorkerIdle,
                glyph: 'i',
                fallback_glyph: 'i',
                layer: 9,
                style_token: StyleToken::Ally,
                priority: 9,
            },
            SymbolDef {
                visual_token: VisualToken::WorkerEnRoute,
                glyph: 'e',
                fallback_glyph: 'e',
                layer: 9,
                style_token: StyleToken::Ally,
                priority: 9,
            },
            SymbolDef {
                visual_token: VisualToken::WorkerWorking,
                glyph: '*',
                fallback_glyph: '*',
                layer: 9,
                style_token: StyleToken::Ally,
                priority: 9,
            },
            SymbolDef {
                visual_token: VisualToken::WorkerBlocked,
                glyph: 'x',
                fallback_glyph: 'x',
                layer: 9,
                style_token: StyleToken::Danger,
                priority: 9,
            },
            SymbolDef {
                visual_token: VisualToken::WorkerResting,
                glyph: 'r',
                fallback_glyph: 'r',
                layer: 9,
                style_token: StyleToken::Ally,
                priority: 9,
            },
            SymbolDef {
                visual_token: VisualToken::WorkerDefending,
                glyph: 'd',
                fallback_glyph: 'd',
                layer: 9,
                style_token: StyleToken::Ally,
                priority: 9,
            },
            SymbolDef {
                visual_token: VisualToken::Floor,
                glyph: '.',
                fallback_glyph: ' ',
                layer: 0,
                style_token: StyleToken::Terrain,
                priority: 0,
            },
            SymbolDef {
                visual_token: VisualToken::Wall,
                glyph: '#',
                fallback_glyph: '#',
                layer: 0,
                style_token: StyleToken::Wall,
                priority: 1,
            },
            SymbolDef {
                visual_token: VisualToken::DoorClosed,
                glyph: '+',
                fallback_glyph: '+',
                layer: 1,
                style_token: StyleToken::Muted,
                priority: 2,
            },
            SymbolDef {
                visual_token: VisualToken::DoorOpen,
                glyph: '\'',
                fallback_glyph: '\'',
                layer: 0,
                style_token: StyleToken::Terrain,
                priority: 0,
            },
            SymbolDef {
                visual_token: VisualToken::Item,
                glyph: '!',
                fallback_glyph: '?',
                layer: 8,
                style_token: StyleToken::Item,
                priority: 8,
            },
            SymbolDef {
                visual_token: VisualToken::Station,
                glyph: 'S',
                fallback_glyph: 'S',
                layer: 7,
                style_token: StyleToken::Station,
                priority: 7,
            },
            SymbolDef {
                visual_token: VisualToken::ResourceNode,
                glyph: 'R',
                fallback_glyph: 'R',
                layer: 6,
                style_token: StyleToken::ResourceNode,
                priority: 6,
            },
            SymbolDef {
                visual_token: VisualToken::Trees,
                glyph: 'T',
                fallback_glyph: 'T',
                layer: 6,
                style_token: StyleToken::ResourceNode,
                priority: 6,
            },
            SymbolDef {
                visual_token: VisualToken::WaterSource,
                glyph: '~',
                fallback_glyph: '~',
                layer: 6,
                style_token: StyleToken::ResourceNode,
                priority: 6,
            },
            SymbolDef {
                visual_token: VisualToken::WildPlants,
                glyph: 'P',
                fallback_glyph: 'P',
                layer: 6,
                style_token: StyleToken::ResourceNode,
                priority: 6,
            },
            SymbolDef {
                visual_token: VisualToken::TargetIndicator,
                glyph: '→',
                fallback_glyph: '^',
                layer: 8,
                style_token: StyleToken::Selection,
                priority: 9,
            },
            SymbolDef {
                visual_token: VisualToken::Exit,
                glyph: '>',
                fallback_glyph: '>',
                layer: 0,
                style_token: StyleToken::Exit,
                priority: 5,
            },
            SymbolDef {
                visual_token: VisualToken::Selection,
                glyph: 'X',
                fallback_glyph: ' ',
                layer: 100,
                style_token: StyleToken::Selection,
                priority: 100,
            },
            SymbolDef {
                visual_token: VisualToken::InvalidSelection,
                glyph: '!',
                fallback_glyph: '!',
                layer: 100,
                style_token: StyleToken::Danger,
                priority: 100,
            },
            SymbolDef {
                visual_token: VisualToken::Fog,
                glyph: ' ',
                fallback_glyph: ' ',
                layer: 50,
                style_token: StyleToken::Muted,
                priority: 50,
            },
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_token_resolves_to_player_symbol() {
        let reg = SymbolRegistry::phase5_defaults();
        let sym = reg.get(VisualToken::Player).unwrap();
        assert_eq!(sym.glyph, '@');
        assert_eq!(sym.style_token, StyleToken::Player);
    }

    #[test]
    fn wall_token_resolves_to_wall_symbol() {
        let reg = SymbolRegistry::phase5_defaults();
        let sym = reg.get(VisualToken::Wall).unwrap();
        assert_eq!(sym.glyph, '#');
        assert_eq!(sym.style_token, StyleToken::Wall);
    }

    #[test]
    fn player_layer_renders_above_floor() {
        let reg = SymbolRegistry::phase5_defaults();
        let player = reg.get(VisualToken::Player).unwrap();
        let floor = reg.get(VisualToken::Floor).unwrap();
        assert!(player.layer > floor.layer, "Player must render above floor");
    }

    #[test]
    fn enemy_layer_renders_above_item() {
        let reg = SymbolRegistry::phase5_defaults();
        let enemy = reg.get(VisualToken::Enemy).unwrap();
        let item = reg.get(VisualToken::Item).unwrap();
        assert!(enemy.layer > item.layer, "Enemy must render above items");
    }

    #[test]
    fn missing_symbol_def_fails_validation() {
        let reg = SymbolRegistry::new(vec![]); // empty
        let errors = reg.validate();
        assert!(!errors.is_empty(), "Empty registry should fail validation");
    }
}
