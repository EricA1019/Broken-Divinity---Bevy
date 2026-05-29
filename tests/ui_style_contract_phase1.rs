use bevy_egui::egui::Color32;

use broken_divinity::ui::ux_style_contract::{MotionProfile, style_for};

const TITLE_GOLD: Color32 = Color32::from_rgb(228, 190, 72);
const SUBTITLE_GOLD: Color32 = Color32::from_rgb(178, 146, 96);
const ACCENT_CRIMSON: Color32 = Color32::from_rgb(186, 34, 48);
const WARNING_AMBER: Color32 = Color32::from_rgb(220, 160, 50);
const DANGER_CRIMSON_HIGH: Color32 = Color32::from_rgb(232, 54, 68);
const SUCCESS_GREEN: Color32 = Color32::from_rgb(116, 188, 112);
const INFO_WARM_TAN: Color32 = Color32::from_rgb(192, 152, 110);
const PANEL_BG_NEAR_BLACK: Color32 = Color32::from_rgb(8, 4, 4);

const HEADING_TIER: f32 = 20.0;
const BODY_TIER: f32 = 16.0;
const SMALL_TIER: f32 = 12.0;
const BASELINE_SPACING: f32 = 1.0;

#[test]
fn style_contract_matches_visual_palette_tokens() {
    let style = style_for();

    assert_eq!(style.title_color, TITLE_GOLD);
    assert_eq!(style.subtitle_color, SUBTITLE_GOLD);
    assert_eq!(style.accent_color, ACCENT_CRIMSON);
    assert_eq!(style.warn_color, WARNING_AMBER);
    assert_eq!(style.danger_color, DANGER_CRIMSON_HIGH);
    assert_eq!(style.success_color, SUCCESS_GREEN);
    assert_eq!(style.info_color, INFO_WARM_TAN);
    assert_eq!(style.panel_bg, PANEL_BG_NEAR_BLACK);
}

#[test]
fn style_contract_matches_symbol_grammar_tokens() {
    let style = style_for();

    assert_eq!(style.symbols.box_nw, "╔");
    assert_eq!(style.symbols.box_h, "═");
    assert_eq!(style.symbols.box_ne, "╗");
    assert_eq!(style.symbols.box_v, "║");
    assert_eq!(style.symbols.box_sw, "╚");
    assert_eq!(style.symbols.box_se, "╝");
    assert_eq!(style.symbols.divider, "│");
    assert_eq!(style.symbols.primary_bullet, "▶");
    assert_eq!(style.symbols.warning_icon, "!");
    assert_eq!(style.symbols.success_icon, "+");
}

#[test]
fn style_contract_matches_type_and_spacing_tiers() {
    let style = style_for();

    assert_eq!(style.heading_size, HEADING_TIER);
    assert_eq!(style.body_size, BODY_TIER);
    assert_eq!(style.small_size, SMALL_TIER);
    assert_eq!(style.spacing, BASELINE_SPACING);
}

#[test]
fn motion_profile_labels_match_visual_contract() {
    assert_eq!(MotionProfile::Subtle.label(), "Subtle");
    assert_eq!(MotionProfile::Drift.label(), "Pulse");
    assert_eq!(MotionProfile::Scanline.label(), "Scanline");
}
