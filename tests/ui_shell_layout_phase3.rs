use broken_divinity::ui::ux_style_contract::runtime_shell_layout;

const HEADER_TO_CONTENT_SPACING: f32 = 8.0;
const SECTION_TO_SECTION_SPACING: f32 = 6.0;
const ACTION_TO_HINT_SPACING: f32 = 4.0;

#[test]
fn runtime_shell_layout_uses_named_spacing_contracts() {
    let layout = runtime_shell_layout();
    assert_eq!(layout.header_to_content_spacing, HEADER_TO_CONTENT_SPACING);
    assert_eq!(layout.section_to_section_spacing, SECTION_TO_SECTION_SPACING);
    assert_eq!(layout.action_to_hint_spacing, ACTION_TO_HINT_SPACING);
}
