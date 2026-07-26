//! Render grid — intermediate representation between game view and terminal.
//!
//! Game state → RenderCellGrid → terminal draw. No raw glyphs in this module.

use ratatui::style::Style;

use crate::{
    theme::ThemeRegistry,
    visual::{StyleToken, SymbolDef, SymbolRegistry, VisualToken},
};

#[derive(Debug, Clone)]
pub struct RenderCell {
    pub glyph: char,
    pub style: Style,
    pub layer: u8,
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub struct RenderCellGrid {
    pub width: u16,
    pub height: u16,
    cells: Vec<RenderCell>,
}

impl RenderCellGrid {
    pub fn new(
        width: u16,
        height: u16,
        fill_token: VisualToken,
        symbols: &SymbolRegistry,
        theme: &ThemeRegistry,
    ) -> Self {
        let fill_cell = make_cell(fill_token, symbols, theme);
        let cells = vec![fill_cell; width as usize * height as usize];
        Self {
            width,
            height,
            cells,
        }
    }

    pub fn set(
        &mut self,
        x: u16,
        y: u16,
        token: VisualToken,
        symbols: &SymbolRegistry,
        theme: &ThemeRegistry,
    ) {
        if x >= self.width || y >= self.height {
            return;
        }
        let new_cell = make_cell(token, symbols, theme);
        let idx = (y * self.width + x) as usize;
        let existing = &self.cells[idx];
        if new_cell.layer > existing.layer
            || (new_cell.layer == existing.layer && new_cell.priority > existing.priority)
        {
            self.cells[idx] = new_cell;
        }
    }

    /// Set a cell with an explicit glyph character, bypassing token lookup.
    /// Used for enemy type glyphs (r, S, B) and other contextual symbols.
    pub fn set_glyph(
        &mut self,
        x: u16,
        y: u16,
        glyph: char,
        token: VisualToken,
        symbols: &SymbolRegistry,
        theme: &ThemeRegistry,
    ) {
        if x >= self.width || y >= self.height {
            return;
        }
        let mut cell = make_cell(token, symbols, theme);
        cell.glyph = glyph;
        let idx = (y * self.width + x) as usize;
        let existing = &self.cells[idx];
        if cell.layer > existing.layer
            || (cell.layer == existing.layer && cell.priority > existing.priority)
        {
            self.cells[idx] = cell;
        }
    }

    pub fn get(&self, x: u16, y: u16) -> Option<&RenderCell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(&self.cells[(y * self.width + x) as usize])
    }

    pub fn rows(&self) -> impl Iterator<Item = Vec<(u16, u16, char, Style)>> + '_ {
        (0..self.height).map(move |y| {
            (0..self.width)
                .map(move |x| {
                    let cell = self.get(x, y).unwrap();
                    (x, y, cell.glyph, cell.style)
                })
                .collect()
        })
    }
}

fn make_cell(token: VisualToken, symbols: &SymbolRegistry, theme: &ThemeRegistry) -> RenderCell {
    let default = SymbolDef {
        visual_token: token,
        glyph: '?',
        fallback_glyph: '?',
        layer: 0,
        style_token: StyleToken::Default,
        priority: 0,
    };
    let sym = symbols.get(token).unwrap_or(&default);
    RenderCell {
        glyph: sym.glyph,
        style: theme.resolve(sym.style_token),
        layer: sym.layer,
        priority: sym.priority,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_grid() -> RenderCellGrid {
        let symbols = SymbolRegistry::phase5_defaults();
        let theme = ThemeRegistry::phase5_defaults();
        RenderCellGrid::new(10, 10, VisualToken::Floor, &symbols, &theme)
    }

    #[test]
    fn player_overwrites_floor() {
        let symbols = SymbolRegistry::phase5_defaults();
        let theme = ThemeRegistry::phase5_defaults();
        let mut grid = test_grid();
        grid.set(5, 5, VisualToken::Player, &symbols, &theme);
        assert_eq!(grid.get(5, 5).unwrap().glyph, '@');
    }

    #[test]
    fn map_grid_resolves_expected_glyphs() {
        let symbols = SymbolRegistry::phase5_defaults();
        let theme = ThemeRegistry::phase5_defaults();
        let mut grid = RenderCellGrid::new(3, 3, VisualToken::Floor, &symbols, &theme);
        grid.set(0, 0, VisualToken::Wall, &symbols, &theme);
        grid.set(1, 1, VisualToken::Player, &symbols, &theme);
        assert_eq!(grid.get(0, 0).unwrap().glyph, '#');
        assert_eq!(grid.get(1, 1).unwrap().glyph, '@');
        assert_eq!(grid.get(2, 2).unwrap().glyph, '.');
    }
}
