#[cfg(test)]
mod tests {
    use bd_tui::theme::{ThemeDef, ThemeRegistry};
    use bd_tui::visual::{StyleToken, SymbolDef, SymbolRegistry, VisualToken};

    const VALID_SYMBOL_RON: &str = r#"[
        SymbolDef(visual_token: Player, glyph: '@', fallback_glyph: '?', layer: 10, style_token: Player, priority: 10),
    ]"#;

    const VALID_THEME_RON: &str = r#"[
        ThemeDef(style_token: Default, fg: "white", bg: None, bold: false),
    ]"#;

    #[test]
    fn loads_valid_symbol_ron() {
        let symbols: Vec<SymbolDef> = ron::from_str(VALID_SYMBOL_RON).unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].visual_token, VisualToken::Player);
        assert_eq!(symbols[0].glyph, '@');
    }

    #[test]
    fn loads_valid_theme_ron() {
        let themes: Vec<ThemeDef> = ron::from_str(VALID_THEME_RON).unwrap();
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].style_token, StyleToken::Default);
    }

    #[test]
    fn rejects_duplicate_symbol_id() {
        let duplicate = SymbolDef {
            visual_token: VisualToken::Player,
            glyph: '@',
            fallback_glyph: '?',
            layer: 10,
            style_token: StyleToken::Player,
            priority: 10,
        };
        let reg = SymbolRegistry::new(vec![duplicate.clone(), duplicate]);
        let errors = reg.validate();
        assert!(errors.iter().any(|error| error.contains("Duplicate")));
    }

    #[test]
    fn rejects_missing_style_reference() {
        let theme = ThemeRegistry::new(vec![ThemeDef {
            style_token: StyleToken::Player,
            fg: "yellow".into(),
            bg: None,
            bold: true,
        }]);
        let errors = theme.validate();
        assert!(errors.iter().any(|e| e.contains("Default")));
    }
}
