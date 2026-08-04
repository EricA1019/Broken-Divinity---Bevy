//! Reusable visual chrome for every terminal screen.
//!
//! Renderers choose semantic roles. Only the theme registry decides which
//! terminal colors and modifiers represent those roles.

use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders},
};

use crate::{theme::ThemeRegistry, visual::StyleToken};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTone {
    Text,
    Muted,
    Accent,
    Positive,
    Warning,
    Info,
    Danger,
    KeyHint,
    Title,
}

impl UiTone {
    const fn token(self) -> StyleToken {
        match self {
            Self::Text => StyleToken::UiText,
            Self::Muted => StyleToken::UiMuted,
            Self::Accent => StyleToken::UiAccent,
            Self::Positive => StyleToken::UiPositive,
            Self::Warning => StyleToken::UiWarning,
            Self::Info => StyleToken::UiInfo,
            Self::Danger => StyleToken::UiDanger,
            Self::KeyHint => StyleToken::UiKeyHint,
            Self::Title => StyleToken::TitleWordmark,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelTone {
    Standard,
    Modal,
    Danger,
    /// Dense data panels (log, action feed) keep muted single rules so the
    /// double-line Ruined Reliquary frame stays the structural hierarchy.
    Dense,
}

pub fn style(theme: &ThemeRegistry, tone: UiTone) -> Style {
    theme.resolve(tone.token())
}

pub fn panel(theme: &ThemeRegistry, title: impl Into<String>, tone: PanelTone) -> Block<'static> {
    let title = title.into();
    let (border, title_style, border_type, marker) = match tone {
        PanelTone::Standard => (
            StyleToken::UiPanelBorder,
            StyleToken::UiPanelTitle,
            BorderType::Plain,
            "◆",
        ),
        PanelTone::Modal => (
            StyleToken::UiModalBorder,
            StyleToken::UiModalTitle,
            BorderType::Double,
            "◆",
        ),
        PanelTone::Danger => (
            StyleToken::UiDanger,
            StyleToken::UiDanger,
            BorderType::Double,
            "!",
        ),
        PanelTone::Dense => (
            StyleToken::UiPanelBorder,
            StyleToken::UiPanelTitle,
            BorderType::Plain,
            "◆",
        ),
    };
    Block::default()
        .title(Line::styled(
            format!(" {marker} {title} "),
            theme.resolve(title_style),
        ))
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(theme.resolve(border))
}

/// Responsive, plain-ASCII semantic meter used anywhere an exact current/max
/// value benefits from a visible proportion. The numeric value is always kept;
/// the track contracts before information is removed.
pub fn meter(
    theme: &ThemeRegistry,
    label: impl Into<String>,
    current: i32,
    maximum: i32,
    width: u16,
    fill_tone: UiTone,
) -> Line<'static> {
    let label = label.into();
    let value = format!("{current}/{maximum}");
    let spacious_width = label.chars().count() + value.chars().count() + 2;
    let compact_width = label.chars().count() + value.chars().count();
    let available = usize::from(width);
    let spacious = available >= spacious_width + 4;
    let prefix_width = if spacious {
        spacious_width
    } else {
        compact_width
    };
    let track_width = available.saturating_sub(prefix_width + 2).min(8);

    let mut spans = vec![Span::styled(label, style(theme, UiTone::Muted))];
    if spacious {
        spans.push(Span::raw(" "));
    }
    spans.push(Span::styled(value, style(theme, UiTone::Text)));
    if spacious {
        spans.push(Span::raw(" "));
    }

    if track_width >= 2 {
        let mut filled = if maximum <= 0 {
            0
        } else {
            (current.clamp(0, maximum) as usize * track_width) / maximum as usize
        };
        if current > 0 {
            filled = filled.max(1);
        }
        // The shared meter language always keeps one visible remainder cell so
        // a full pool still reads as a bounded track, never a solid block.
        filled = filled.min(track_width - 1);
        spans.push(Span::styled("[", style(theme, UiTone::Accent)));
        spans.push(Span::styled("#".repeat(filled), style(theme, fill_tone)));
        spans.push(Span::styled(
            "-".repeat(track_width - filled),
            style(theme, UiTone::Muted),
        ));
        spans.push(Span::styled("]", style(theme, UiTone::Accent)));
    }

    Line::from(spans)
}

/// Exact-value resource gauge: semantic label, exact current stock, and a
/// responsive partial track. The bound and fill token are projected facts
/// (never a renderer constant) and the exact stock is never hidden to make
/// room. Any pool can reuse this primitive; it is not Supplies-specific.
pub fn resource_gauge(
    theme: &ThemeRegistry,
    label: impl Into<String>,
    current: i32,
    maximum: i32,
    width: u16,
    fill_token: StyleToken,
) -> Line<'static> {
    let label = label.into();
    let value = format!("{current}");
    let spacious_width = label.chars().count() + value.chars().count() + 2;
    let compact_width = label.chars().count() + value.chars().count();
    let available = usize::from(width);
    // Prefer a spacious layout only when the resulting track still has room to
    // show a meaningful proportion (at least four cells).
    let spacious = available >= spacious_width + 6;
    let prefix_width = if spacious {
        spacious_width
    } else {
        compact_width
    };
    let track_width = available.saturating_sub(prefix_width + 2).min(8);

    let mut spans = vec![Span::styled(label, theme.resolve(StyleToken::UiMuted))];
    if spacious {
        spans.push(Span::raw(" "));
    }
    spans.push(Span::styled(value, theme.resolve(StyleToken::UiText)));
    if spacious {
        spans.push(Span::raw(" "));
    }

    if track_width >= 2 {
        let mut filled = if maximum <= 0 {
            0
        } else {
            (current.clamp(0, maximum) as usize * track_width) / maximum as usize
        };
        if current > 0 {
            filled = filled.max(1);
        }
        filled = filled.min(track_width - 1);
        spans.push(Span::styled("[", theme.resolve(StyleToken::UiAccent)));
        spans.push(Span::styled("#".repeat(filled), theme.resolve(fill_token)));
        spans.push(Span::styled(
            "-".repeat(track_width - filled),
            theme.resolve(StyleToken::UiMuted),
        ));
        spans.push(Span::styled("]", theme.resolve(StyleToken::UiAccent)));
    }

    Line::from(spans)
}

/// Screen/day/turn ribbon. Presentation is shared across colony, dungeon,
/// inventory, and modal interaction states; callers provide only the semantic
/// mode label and values.
pub fn mode_ribbon(
    theme: &ThemeRegistry,
    mode_label: impl Into<String>,
    day: u64,
    turn: u64,
    version: &str,
) -> Line<'static> {
    Line::from(vec![
        Span::styled("◆ ", style(theme, UiTone::Accent)),
        Span::styled(mode_label.into(), style(theme, UiTone::Title)),
        Span::styled("  ·  ", style(theme, UiTone::Accent)),
        Span::styled(format!("DAY {day:02}"), style(theme, UiTone::Text)),
        Span::styled("  ·  ", style(theme, UiTone::Accent)),
        Span::styled(format!("TURN {turn}"), style(theme, UiTone::Text)),
        Span::styled("  ·  ", style(theme, UiTone::Accent)),
        Span::styled(format!("KERNEL v{version}"), style(theme, UiTone::Muted)),
    ])
}

fn looks_like_key(value: &str) -> bool {
    value == "Enter"
        || value == "Esc"
        || value.contains("Esc")
        || value.contains('-')
        || value.contains('/')
        || value.chars().all(|character| {
            character.is_ascii_digit() || character.is_ascii_uppercase() || character == '?'
        })
}

/// Format semantic `Label:key` footer projections as a compact command ribbon.
/// Packing is presentation-only: command ownership remains in `commands`.
pub fn command_ribbon(theme: &ThemeRegistry, groups: &[&str], width: u16) -> Line<'static> {
    let tokens = groups
        .iter()
        .flat_map(|group| group.split(" | "))
        .filter(|token| !token.is_empty())
        .filter_map(|token| {
            let (left, right) = token.split_once(':')?;
            let (key, label) = if looks_like_key(left) {
                (left, right)
            } else {
                (right, left)
            };
            Some((key, label, key.chars().count() + label.chars().count() + 3))
        })
        .collect::<Vec<_>>();
    let generous_width = tokens.iter().map(|(_, _, width)| width).sum::<usize>()
        + tokens.len().saturating_sub(1) * 2;
    let separator = if generous_width <= usize::from(width) {
        "  "
    } else {
        " "
    };
    let mut spans = Vec::new();
    let mut used = 0_usize;

    for (key, label, token_width) in tokens {
        let separator_width = usize::from(!spans.is_empty()) * separator.len();
        if used + separator_width + token_width > usize::from(width) {
            break;
        }
        if !spans.is_empty() {
            spans.push(Span::styled(separator, style(theme, UiTone::Muted)));
            used += separator.len();
        }
        spans.push(Span::styled(
            format!("[{key}]"),
            style(theme, UiTone::KeyHint),
        ));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(label.to_owned(), style(theme, UiTone::Text)));
        used += token_width;
    }

    Line::from(spans)
}

/// Structural Ruined Reliquary frame drawn around the entire terminal so every
/// colony and reusable screen carries one closed double-line frame.
///
/// Renders a continuous double-line perimeter (`╔╗╚╝` corners, `═` top/bottom
/// rules, `║` side rails) owned by the primary frame role. It is painted last
/// (after panels and footer) so no content can overwrite a structural edge
/// cell; overlays rendered after it may intentionally cover edge cells.
pub fn render_outer_frame(frame: &mut ratatui::Frame, theme: &ThemeRegistry) {
    let area = frame.area();
    if area.width < 2 || area.height < 2 {
        return;
    }
    let style = theme.resolve(StyleToken::UiModalBorder);
    let first_x = area.x;
    let last_x = area.x + area.width - 1;
    let first_y = area.y;
    let last_y = area.y + area.height - 1;
    let buffer = frame.buffer_mut();
    for x in first_x..=last_x {
        let (top, bottom) = if x == first_x {
            ("╔", "╚")
        } else if x == last_x {
            ("╗", "╝")
        } else {
            ("═", "═")
        };
        buffer[(x, first_y)].set_symbol(top).set_style(style);
        buffer[(x, last_y)].set_symbol(bottom).set_style(style);
    }
    for y in (first_y + 1)..last_y {
        buffer[(first_x, y)].set_symbol("║").set_style(style);
        buffer[(last_x, y)].set_symbol("║").set_style(style);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        style::{Color, Modifier},
        widgets::Widget,
    };

    use super::*;

    #[test]
    fn standard_panel_separates_neutral_border_from_emphasized_title() {
        let theme = ThemeRegistry::phase5_defaults();
        let expected_border = theme.resolve(StyleToken::UiPanelBorder);
        let expected_title = theme.resolve(StyleToken::UiPanelTitle);
        let block = panel(&theme, "Party", PanelTone::Standard);
        let area = Rect::new(0, 0, 16, 3);
        let mut buffer = Buffer::empty(area);
        block.render(area, &mut buffer);

        let border = buffer.cell((0, 0)).expect("top-left border must render");
        assert_eq!(border.fg, expected_border.fg.unwrap_or(Color::Reset));
        assert_eq!(
            border.symbol(),
            "┌",
            "ordinary panels use muted single rules"
        );
        let marker = buffer.cell((2, 0)).expect("title marker must render");
        assert_eq!(marker.symbol(), "◆");
        let title = (1..15)
            .filter_map(|x| buffer.cell((x, 0)))
            .find(|cell| cell.symbol() == "P")
            .expect("title text must render");
        assert_eq!(title.fg, expected_title.fg.unwrap_or(Color::Reset));
        assert_ne!(
            border.fg, title.fg,
            "standard panel border and title must retain distinct hierarchy"
        );
        assert!(
            title.modifier.contains(Modifier::BOLD),
            "panel title should own stronger visual hierarchy"
        );
    }

    #[test]
    fn semantic_tones_resolve_without_renderer_owned_colors() {
        let theme = ThemeRegistry::phase5_defaults();
        let positive = style(&theme, UiTone::Positive);
        let warning = style(&theme, UiTone::Warning);
        let danger = style(&theme, UiTone::Danger);

        assert!(positive.fg.is_some());
        assert!(warning.fg.is_some());
        assert!(danger.fg.is_some());
        assert_ne!(positive.fg, warning.fg);
        assert_ne!(positive.fg, danger.fg);
        assert_ne!(warning.fg, danger.fg);
        assert!(danger.add_modifier.contains(Modifier::BOLD));
    }
}
