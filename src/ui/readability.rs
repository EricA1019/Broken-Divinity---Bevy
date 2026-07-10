pub(crate) fn contrast_ratio(foreground_rgb: (u8, u8, u8), background_rgb: (u8, u8, u8)) -> f32 {
    let fg_luminance = relative_luminance(foreground_rgb);
    let bg_luminance = relative_luminance(background_rgb);
    let (lighter, darker) = if fg_luminance >= bg_luminance {
        (fg_luminance, bg_luminance)
    } else {
        (bg_luminance, fg_luminance)
    };

    (lighter + LUMINANCE_FLOOR_OFFSET) / (darker + LUMINANCE_FLOOR_OFFSET)
}

const LUMINANCE_FLOOR_OFFSET: f32 = 0.05;
const SRGB_NORMALIZATION_SCALE: f32 = 255.0;
const LINEARIZATION_THRESHOLD: f32 = 0.039_28;
const LINEARIZATION_DIVISOR: f32 = 12.92;
const LINEARIZATION_OFFSET: f32 = 0.055;
const LINEARIZATION_FACTOR: f32 = 1.055;
const LINEARIZATION_EXPONENT: f32 = 2.4;
const RED_LUMA_WEIGHT: f32 = 0.2126;
const GREEN_LUMA_WEIGHT: f32 = 0.7152;
const BLUE_LUMA_WEIGHT: f32 = 0.0722;

fn relative_luminance((red, green, blue): (u8, u8, u8)) -> f32 {
    let red_linear = channel_to_linear(red);
    let green_linear = channel_to_linear(green);
    let blue_linear = channel_to_linear(blue);

    (RED_LUMA_WEIGHT * red_linear)
        + (GREEN_LUMA_WEIGHT * green_linear)
        + (BLUE_LUMA_WEIGHT * blue_linear)
}

fn channel_to_linear(channel: u8) -> f32 {
    let normalized = f32::from(channel) / SRGB_NORMALIZATION_SCALE;
    if normalized <= LINEARIZATION_THRESHOLD {
        normalized / LINEARIZATION_DIVISOR
    } else {
        ((normalized + LINEARIZATION_OFFSET) / LINEARIZATION_FACTOR)
            .powf(LINEARIZATION_EXPONENT)
    }
}

#[cfg(test)]
mod tests {
    use super::contrast_ratio;

    #[test]
    fn contrast_ratio_black_on_white_is_high() {
        let ratio = contrast_ratio((0, 0, 0), (255, 255, 255));
        assert!(ratio > 20.0, "Expected black on white contrast near WCAG max");
    }

    #[test]
    fn contrast_ratio_identical_colors_is_minimum() {
        let ratio = contrast_ratio((40, 40, 40), (40, 40, 40));
        assert!(ratio < 1.1, "Expected same-color contrast near 1.0");
    }
}
