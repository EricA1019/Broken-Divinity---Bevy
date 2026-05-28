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
            Self::Drift => "Drift",
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

pub fn style_for() -> VariantStyle {
    VariantStyle {
        symbols: SymbolSet {
            box_nw: "╭",
            box_h: "─",
            box_ne: "╮",
            box_v: "│",
            box_sw: "╰",
            box_se: "╯",
            divider: "┆",
            primary_bullet: "▸",
            warning_icon: "⚠",
            success_icon: "●",
        },
        title_color: egui::Color32::from_rgb(238, 197, 82),
        subtitle_color: egui::Color32::from_rgb(174, 145, 102),
        accent_color: egui::Color32::from_rgb(198, 44, 52),
        warn_color: egui::Color32::from_rgb(230, 176, 74),
        danger_color: egui::Color32::from_rgb(239, 72, 84),
        success_color: egui::Color32::from_rgb(112, 198, 150),
        info_color: egui::Color32::from_rgb(164, 186, 147),
        panel_bg: egui::Color32::from_rgb(6, 9, 10),
        mono_all: true,
        decorated_borders: true,
        spacing: 0.98,
        heading_size: 20.0,
        body_size: 16.0,
        small_size: 12.0,
    }
}
