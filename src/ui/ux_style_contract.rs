use bevy_egui::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionProfile {
    Subtle,
    Drift,
    Scanline,
}

impl MotionProfile {
    pub fn label(self) -> &'static str {
        match self {
            Self::Subtle => "Subtle",
            Self::Drift => "Pulse",
            Self::Scanline => "Scanline",
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct SymbolSet {
    pub box_nw: &'static str,
    pub box_h: &'static str,
    pub box_ne: &'static str,
    pub box_v: &'static str,
    pub box_sw: &'static str,
    pub box_se: &'static str,
    pub divider: &'static str,
    pub primary_bullet: &'static str,
    pub warning_icon: &'static str,
    pub success_icon: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct VariantStyle {
    pub symbols: SymbolSet,
    pub title_color: egui::Color32,
    pub subtitle_color: egui::Color32,
    pub accent_color: egui::Color32,
    pub warn_color: egui::Color32,
    pub danger_color: egui::Color32,
    pub success_color: egui::Color32,
    pub info_color: egui::Color32,
    pub panel_bg: egui::Color32,
    pub mono_all: bool,
    pub decorated_borders: bool,
    pub spacing: f32,
    pub heading_size: f32,
    pub body_size: f32,
    pub small_size: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct RuntimeStyleAdapter {
    pub menu_background_rgb: (u8, u8, u8),
    pub menu_title_rgb: (u8, u8, u8),
    pub menu_subtitle_rgb: (u8, u8, u8),
    pub menu_seed_label_rgb: (u8, u8, u8),
    pub menu_min_contrast_ratio: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct RuntimeShellLayout {
    pub header_to_content_spacing: f32,
    pub section_to_section_spacing: f32,
    pub action_to_hint_spacing: f32,
}

const MENU_MIN_CONTRAST_RATIO: f32 = 4.5;
const HEADER_TO_CONTENT_SPACING: f32 = 8.0;
const SECTION_TO_SECTION_SPACING: f32 = 6.0;
const ACTION_TO_HINT_SPACING: f32 = 4.0;

pub fn runtime_style_adapter() -> RuntimeStyleAdapter {
    let style = style_for();
    RuntimeStyleAdapter {
        menu_background_rgb: rgb_tuple(style.panel_bg),
        menu_title_rgb: rgb_tuple(style.title_color),
        menu_subtitle_rgb: rgb_tuple(style.subtitle_color),
        menu_seed_label_rgb: rgb_tuple(style.info_color),
        menu_min_contrast_ratio: MENU_MIN_CONTRAST_RATIO,
    }
}

pub fn runtime_shell_layout() -> RuntimeShellLayout {
    RuntimeShellLayout {
        header_to_content_spacing: HEADER_TO_CONTENT_SPACING,
        section_to_section_spacing: SECTION_TO_SECTION_SPACING,
        action_to_hint_spacing: ACTION_TO_HINT_SPACING,
    }
}

fn rgb_tuple(color: egui::Color32) -> (u8, u8, u8) {
    (color.r(), color.g(), color.b())
}

pub fn style_for() -> VariantStyle {
    VariantStyle {
        symbols: SymbolSet {
            box_nw: "╔",
            box_h: "═",
            box_ne: "╗",
            box_v: "║",
            box_sw: "╚",
            box_se: "╝",
            divider: "│",
            primary_bullet: "▶",
            warning_icon: "!",
            success_icon: "+",
        },
        title_color: egui::Color32::from_rgb(228, 190, 72),
        subtitle_color: egui::Color32::from_rgb(178, 146, 96),
        accent_color: egui::Color32::from_rgb(186, 34, 48),
        warn_color: egui::Color32::from_rgb(220, 160, 50),
        danger_color: egui::Color32::from_rgb(232, 54, 68),
        success_color: egui::Color32::from_rgb(116, 188, 112),
        info_color: egui::Color32::from_rgb(192, 152, 110),
        panel_bg: egui::Color32::from_rgb(8, 4, 4),
        mono_all: true,
        decorated_borders: true,
        spacing: 1.0,
        heading_size: 20.0,
        body_size: 16.0,
        small_size: 12.0,
    }
}
