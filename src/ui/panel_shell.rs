use bevy_egui::egui;

use crate::ui::ux_style_contract::{runtime_shell_layout, style_for};

const SHEET_STROKE_WIDTH: f32 = 1.0;
const SHEET_STROKE_ALPHA: f32 = 0.35;
const SHEET_PANEL_EXTRA_MARGIN_X: i8 = 10;
const SHEET_PANEL_EXTRA_MARGIN_Y: i8 = 8;
const STRIP_PANEL_EXTRA_MARGIN_X: i8 = 8;
const STRIP_PANEL_EXTRA_MARGIN_Y: i8 = 6;

pub fn sheet_window(title: &'static str) -> egui::Window<'static> {
    egui::Window::new(title).frame(sheet_frame())
}

pub fn sheet_frame() -> egui::Frame {
    let style = style_for();
    let shell_layout = runtime_shell_layout();

    egui::Frame::NONE
        .fill(style.panel_bg)
        .stroke(egui::Stroke::new(
            SHEET_STROKE_WIDTH,
            style.title_color.gamma_multiply(SHEET_STROKE_ALPHA),
        ))
        .inner_margin(egui::Margin::symmetric(
            shell_layout.header_to_content_spacing as i8 + SHEET_PANEL_EXTRA_MARGIN_X,
            shell_layout.action_to_hint_spacing as i8 + SHEET_PANEL_EXTRA_MARGIN_Y,
        ))
}

pub fn strip_frame() -> egui::Frame {
    let style = style_for();
    let shell_layout = runtime_shell_layout();

    egui::Frame::NONE
        .fill(style.panel_bg)
        .inner_margin(egui::Margin::symmetric(
            shell_layout.header_to_content_spacing as i8 + STRIP_PANEL_EXTRA_MARGIN_X,
            shell_layout.action_to_hint_spacing as i8 + STRIP_PANEL_EXTRA_MARGIN_Y,
        ))
}