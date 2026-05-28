const GAMMA_THRESHOLD: f32 = 0.04045;
const LINEAR_DIVISOR: f32 = 12.92;
const GAMMA_OFFSET: f32 = 0.055;
const GAMMA_SCALE: f32 = 1.055;
const GAMMA_EXPONENT: f32 = 2.4;
const RED_WEIGHT: f32 = 0.2126;
const GREEN_WEIGHT: f32 = 0.7152;
const BLUE_WEIGHT: f32 = 0.0722;
const LUMINANCE_OFFSET: f32 = 0.05;

pub fn contrast_ratio(foreground: (u8, u8, u8), background: (u8, u8, u8)) -> f32 {
    let foreground_luminance = relative_luminance(foreground);
    let background_luminance = relative_luminance(background);

    let lighter = foreground_luminance.max(background_luminance);
    let darker = foreground_luminance.min(background_luminance);

    (lighter + LUMINANCE_OFFSET) / (darker + LUMINANCE_OFFSET)
}

fn relative_luminance(rgb: (u8, u8, u8)) -> f32 {
    let (red, green, blue) = rgb;

    linearize(red) * RED_WEIGHT + linearize(green) * GREEN_WEIGHT + linearize(blue) * BLUE_WEIGHT
}

fn linearize(channel: u8) -> f32 {
    let normalized = channel as f32 / u8::MAX as f32;
    if normalized <= GAMMA_THRESHOLD {
        return normalized / LINEAR_DIVISOR;
    }

    ((normalized + GAMMA_OFFSET) / GAMMA_SCALE).powf(GAMMA_EXPONENT)
}
