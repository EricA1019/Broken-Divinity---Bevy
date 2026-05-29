use broken_divinity::ui::ux_style_contract::{runtime_style_adapter, style_for};

const MENU_MIN_CONTRAST_RATIO: f32 = 4.5;

#[test]
fn runtime_style_adapter_derives_menu_colors_from_style_contract() {
    let style = style_for();
    let adapter = runtime_style_adapter();

    assert_eq!(adapter.menu_background_rgb, (8, 4, 4));
    assert_eq!(adapter.menu_title_rgb, (228, 190, 72));
    assert_eq!(adapter.menu_subtitle_rgb, (178, 146, 96));
    assert_eq!(adapter.menu_seed_label_rgb, (192, 152, 110));

    assert_eq!(adapter.menu_background_rgb, (style.panel_bg.r(), style.panel_bg.g(), style.panel_bg.b()));
    assert_eq!(adapter.menu_title_rgb, (style.title_color.r(), style.title_color.g(), style.title_color.b()));
    assert_eq!(adapter.menu_subtitle_rgb, (style.subtitle_color.r(), style.subtitle_color.g(), style.subtitle_color.b()));
    assert_eq!(adapter.menu_seed_label_rgb, (style.info_color.r(), style.info_color.g(), style.info_color.b()));
}

#[test]
fn runtime_style_adapter_keeps_readability_threshold_contract() {
    let adapter = runtime_style_adapter();
    assert_eq!(adapter.menu_min_contrast_ratio, MENU_MIN_CONTRAST_RATIO);
}
