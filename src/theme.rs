// theme.rs

use std::time::Duration;

use crate::UserPreferences;

pub const MINIMUM_TEXT_CONTRAST: f32 = 4.5;
pub const MINIMUM_UI_CONTRAST: f32 = 3.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LinearColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl LinearColor {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red: normalize_channel(red),
            green: normalize_channel(green),
            blue: normalize_channel(blue),
            alpha: normalize_channel(alpha),
        }
    }

    pub fn from_srgb(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self::new(
            srgb_to_linear(normalize_channel(red)),
            srgb_to_linear(normalize_channel(green)),
            srgb_to_linear(normalize_channel(blue)),
            alpha,
        )
    }

    pub fn from_srgb8(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self::from_srgb(
            f32::from(red) / 255.0,
            f32::from(green) / 255.0,
            f32::from(blue) / 255.0,
            f32::from(alpha) / 255.0,
        )
    }

    pub fn to_array(self) -> [f32; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }

    pub fn to_srgb_array(self) -> [f32; 4] {
        [
            linear_to_srgb(self.red),
            linear_to_srgb(self.green),
            linear_to_srgb(self.blue),
            self.alpha,
        ]
    }

    pub fn relative_luminance(self) -> f32 {
        0.2126 * self.red + 0.7152 * self.green + 0.0722 * self.blue
    }

    pub fn contrast_ratio(self, other: Self) -> f32 {
        let lighter = self.relative_luminance().max(other.relative_luminance());
        let darker = self.relative_luminance().min(other.relative_luminance());
        (lighter + 0.05) / (darker + 0.05)
    }

    pub fn composite_over(self, background: Self) -> Self {
        let alpha = self.alpha + background.alpha * (1.0 - self.alpha);
        if alpha == 0.0 {
            return Self::default();
        }
        Self::new(
            (self.red * self.alpha + background.red * background.alpha * (1.0 - self.alpha))
                / alpha,
            (self.green * self.alpha + background.green * background.alpha * (1.0 - self.alpha))
                / alpha,
            (self.blue * self.alpha + background.blue * background.alpha * (1.0 - self.alpha))
                / alpha,
            alpha,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SemanticColorToken {
    Background,
    Surface,
    SurfaceElevated,
    Text,
    TextMuted,
    Primary,
    OnPrimary,
    Border,
    Focus,
    Error,
    OnError,
    Success,
    Warning,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SemanticColors {
    pub background: LinearColor,
    pub surface: LinearColor,
    pub surface_elevated: LinearColor,
    pub text: LinearColor,
    pub text_muted: LinearColor,
    pub primary: LinearColor,
    pub on_primary: LinearColor,
    pub border: LinearColor,
    pub focus: LinearColor,
    pub error: LinearColor,
    pub on_error: LinearColor,
    pub success: LinearColor,
    pub warning: LinearColor,
}

impl SemanticColors {
    pub fn resolve(self, token: SemanticColorToken) -> LinearColor {
        match token {
            SemanticColorToken::Background => self.background,
            SemanticColorToken::Surface => self.surface,
            SemanticColorToken::SurfaceElevated => self.surface_elevated,
            SemanticColorToken::Text => self.text,
            SemanticColorToken::TextMuted => self.text_muted,
            SemanticColorToken::Primary => self.primary,
            SemanticColorToken::OnPrimary => self.on_primary,
            SemanticColorToken::Border => self.border,
            SemanticColorToken::Focus => self.focus,
            SemanticColorToken::Error => self.error,
            SemanticColorToken::OnError => self.on_error,
            SemanticColorToken::Success => self.success,
            SemanticColorToken::Warning => self.warning,
        }
    }

    pub fn light() -> Self {
        Self {
            background: LinearColor::from_srgb8(250, 250, 250, 255),
            surface: LinearColor::from_srgb8(255, 255, 255, 255),
            surface_elevated: LinearColor::from_srgb8(255, 255, 255, 255),
            text: LinearColor::from_srgb8(31, 31, 35, 255),
            text_muted: LinearColor::from_srgb8(92, 92, 102, 255),
            primary: LinearColor::from_srgb8(180, 83, 9, 255),
            on_primary: LinearColor::from_srgb8(255, 255, 255, 255),
            border: LinearColor::from_srgb8(209, 209, 216, 255),
            focus: LinearColor::from_srgb8(180, 83, 9, 255),
            error: LinearColor::from_srgb8(185, 28, 28, 255),
            on_error: LinearColor::from_srgb8(255, 255, 255, 255),
            success: LinearColor::from_srgb8(21, 128, 61, 255),
            warning: LinearColor::from_srgb8(180, 83, 9, 255),
        }
    }

    pub fn dark() -> Self {
        Self {
            background: LinearColor::from_srgb8(24, 24, 27, 255),
            surface: LinearColor::from_srgb8(39, 39, 42, 255),
            surface_elevated: LinearColor::from_srgb8(63, 63, 70, 255),
            text: LinearColor::from_srgb8(250, 250, 250, 255),
            text_muted: LinearColor::from_srgb8(180, 180, 190, 255),
            primary: LinearColor::from_srgb8(245, 158, 11, 255),
            on_primary: LinearColor::from_srgb8(31, 31, 35, 255),
            border: LinearColor::from_srgb8(82, 82, 91, 255),
            focus: LinearColor::from_srgb8(251, 191, 36, 255),
            error: LinearColor::from_srgb8(248, 113, 113, 255),
            on_error: LinearColor::from_srgb8(31, 31, 35, 255),
            success: LinearColor::from_srgb8(74, 222, 128, 255),
            warning: LinearColor::from_srgb8(251, 191, 36, 255),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ColorScheme {
    #[default]
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
    #[default]
    System,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemePair {
    pub light: SemanticColors,
    pub dark: SemanticColors,
}

impl Default for ThemePair {
    fn default() -> Self {
        Self {
            light: SemanticColors::light(),
            dark: SemanticColors::dark(),
        }
    }
}

impl ThemePair {
    pub fn resolve(self, scheme: ColorScheme) -> SemanticColors {
        match scheme {
            ColorScheme::Light => self.light,
            ColorScheme::Dark => self.dark,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContrastThemePairs {
    pub normal: ThemePair,
    pub more: ThemePair,
    pub less: ThemePair,
}

impl Default for ContrastThemePairs {
    fn default() -> Self {
        let normal = ThemePair::default();
        Self {
            normal,
            more: ThemePair {
                light: SemanticColors {
                    background: LinearColor::from_srgb8(255, 255, 255, 255),
                    surface: LinearColor::from_srgb8(255, 255, 255, 255),
                    surface_elevated: LinearColor::from_srgb8(255, 255, 255, 255),
                    text: LinearColor::from_srgb8(0, 0, 0, 255),
                    text_muted: LinearColor::from_srgb8(64, 64, 64, 255),
                    primary: LinearColor::from_srgb8(146, 64, 14, 255),
                    on_primary: LinearColor::from_srgb8(255, 255, 255, 255),
                    border: LinearColor::from_srgb8(82, 82, 82, 255),
                    focus: LinearColor::from_srgb8(146, 64, 14, 255),
                    error: LinearColor::from_srgb8(153, 27, 27, 255),
                    on_error: LinearColor::from_srgb8(255, 255, 255, 255),
                    success: LinearColor::from_srgb8(20, 83, 45, 255),
                    warning: LinearColor::from_srgb8(146, 64, 14, 255),
                },
                dark: SemanticColors {
                    background: LinearColor::from_srgb8(0, 0, 0, 255),
                    surface: LinearColor::from_srgb8(24, 24, 27, 255),
                    surface_elevated: LinearColor::from_srgb8(39, 39, 42, 255),
                    text: LinearColor::from_srgb8(255, 255, 255, 255),
                    text_muted: LinearColor::from_srgb8(212, 212, 216, 255),
                    primary: LinearColor::from_srgb8(251, 191, 36, 255),
                    on_primary: LinearColor::from_srgb8(0, 0, 0, 255),
                    border: LinearColor::from_srgb8(161, 161, 170, 255),
                    focus: LinearColor::from_srgb8(252, 211, 77, 255),
                    error: LinearColor::from_srgb8(252, 165, 165, 255),
                    on_error: LinearColor::from_srgb8(0, 0, 0, 255),
                    success: LinearColor::from_srgb8(134, 239, 172, 255),
                    warning: LinearColor::from_srgb8(252, 211, 77, 255),
                },
            },
            less: ThemePair {
                light: SemanticColors {
                    text: LinearColor::from_srgb8(82, 82, 91, 255),
                    text_muted: LinearColor::from_srgb8(113, 113, 122, 255),
                    border: LinearColor::from_srgb8(228, 228, 231, 255),
                    ..normal.light
                },
                dark: SemanticColors {
                    text: LinearColor::from_srgb8(228, 228, 231, 255),
                    text_muted: LinearColor::from_srgb8(161, 161, 170, 255),
                    border: LinearColor::from_srgb8(82, 82, 91, 255),
                    ..normal.dark
                },
            },
        }
    }
}

impl ContrastThemePairs {
    pub fn resolve(self, scheme: ColorScheme, preferences: UserPreferences) -> SemanticColors {
        preferences
            .resolve_contrast(self.normal, self.more, self.less)
            .resolve(scheme)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThemeController {
    mode: ThemeMode,
    system_scheme: ColorScheme,
    generation: u64,
}

impl Default for ThemeController {
    fn default() -> Self {
        Self {
            mode: ThemeMode::System,
            system_scheme: ColorScheme::Light,
            generation: 0,
        }
    }
}

impl ThemeController {
    pub fn mode(self) -> ThemeMode {
        self.mode
    }

    pub fn effective_scheme(self) -> ColorScheme {
        match self.mode {
            ThemeMode::Light => ColorScheme::Light,
            ThemeMode::Dark => ColorScheme::Dark,
            ThemeMode::System => self.system_scheme,
        }
    }

    pub fn generation(self) -> u64 {
        self.generation
    }

    pub fn set_mode(&mut self, mode: ThemeMode) -> bool {
        let previous = self.effective_scheme();
        self.mode = mode;
        self.record_effective_change(previous)
    }

    pub fn set_system_scheme(&mut self, scheme: ColorScheme) -> bool {
        let previous = self.effective_scheme();
        self.system_scheme = scheme;
        self.record_effective_change(previous)
    }

    pub fn colors(self, themes: ThemePair) -> SemanticColors {
        themes.resolve(self.effective_scheme())
    }

    fn record_effective_change(&mut self, previous: ColorScheme) -> bool {
        let changed = previous != self.effective_scheme();
        if changed {
            self.generation = self.generation.wrapping_add(1);
        }
        changed
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpacingTokens {
    pub extra_small: f32,
    pub small: f32,
    pub medium: f32,
    pub large: f32,
    pub extra_large: f32,
}

impl SpacingTokens {
    pub fn new(values: [f32; 5]) -> Self {
        let values = values.map(finite_non_negative);
        Self {
            extra_small: values[0],
            small: values[1],
            medium: values[2],
            large: values[3],
            extra_large: values[4],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RadiusTokens {
    pub small: f32,
    pub medium: f32,
    pub large: f32,
    pub pill: f32,
}

impl RadiusTokens {
    pub fn new(small: f32, medium: f32, large: f32, pill: f32) -> Self {
        Self {
            small: finite_non_negative(small),
            medium: finite_non_negative(medium),
            large: finite_non_negative(large),
            pill: finite_non_negative(pill),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypographyToken {
    pub family: String,
    pub size: f32,
    pub weight: u16,
    pub line_height: f32,
    pub letter_spacing: f32,
}

impl TypographyToken {
    pub fn new(
        family: impl Into<String>,
        size: f32,
        weight: u16,
        line_height: f32,
        letter_spacing: f32,
    ) -> Self {
        let size = finite_non_negative(size);
        Self {
            family: family.into(),
            size,
            weight: weight.clamp(1, 1_000),
            line_height: finite_non_negative(line_height).max(size),
            letter_spacing: finite_signed(letter_spacing),
        }
    }

    pub fn resolved_size(&self, preferences: UserPreferences) -> f32 {
        preferences.scaled_text_size(self.size)
    }

    pub fn resolved(&self, preferences: UserPreferences) -> Self {
        let scale = preferences.text_scale();
        Self {
            family: self.family.clone(),
            size: self.size * scale,
            weight: self.weight,
            line_height: self.line_height * scale,
            letter_spacing: self.letter_spacing * scale,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BorderTokens {
    pub thin: f32,
    pub regular: f32,
    pub thick: f32,
}

impl BorderTokens {
    pub fn new(thin: f32, regular: f32, thick: f32) -> Self {
        Self {
            thin: finite_non_negative(thin),
            regular: finite_non_negative(regular),
            thick: finite_non_negative(thick),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShadowToken {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: LinearColor,
}

impl ShadowToken {
    pub fn new(offset_x: f32, offset_y: f32, blur: f32, spread: f32, color: LinearColor) -> Self {
        Self {
            offset_x: finite_signed(offset_x),
            offset_y: finite_signed(offset_y),
            blur: finite_non_negative(blur),
            spread: finite_signed(spread),
            color,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ElevationTokens {
    pub low: ShadowToken,
    pub medium: ShadowToken,
    pub high: ShadowToken,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MotionTokens {
    pub fast: Duration,
    pub normal: Duration,
    pub slow: Duration,
}

impl MotionTokens {
    pub fn resolved(self, preferences: UserPreferences) -> Self {
        Self {
            fast: preferences.motion_duration(self.fast),
            normal: preferences.motion_duration(self.normal),
            slow: preferences.motion_duration(self.slow),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ThemeDefinition {
    pub colors: ContrastThemePairs,
    pub typography: TypographyToken,
    pub spacing: SpacingTokens,
    pub radii: RadiusTokens,
    pub borders: BorderTokens,
    pub elevation: ElevationTokens,
    pub motion: MotionTokens,
}

impl Default for ThemeDefinition {
    fn default() -> Self {
        Self {
            colors: ContrastThemePairs::default(),
            typography: TypographyToken::new("Vazirmatn", 16.0, 400, 24.0, 0.0),
            spacing: SpacingTokens::new([4.0, 8.0, 12.0, 16.0, 24.0]),
            radii: RadiusTokens::new(6.0, 10.0, 16.0, 999.0),
            borders: BorderTokens::new(1.0, 1.5, 2.0),
            elevation: ElevationTokens {
                low: ShadowToken::new(0.0, 1.0, 3.0, 0.0, LinearColor::new(0.0, 0.0, 0.0, 0.12)),
                medium: ShadowToken::new(
                    0.0,
                    4.0,
                    12.0,
                    0.0,
                    LinearColor::new(0.0, 0.0, 0.0, 0.16),
                ),
                high: ShadowToken::new(0.0, 12.0, 28.0, 0.0, LinearColor::new(0.0, 0.0, 0.0, 0.2)),
            },
            motion: MotionTokens {
                fast: Duration::from_millis(100),
                normal: Duration::from_millis(180),
                slow: Duration::from_millis(300),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedTheme {
    pub scheme: ColorScheme,
    pub colors: SemanticColors,
    pub typography: TypographyToken,
    pub spacing: SpacingTokens,
    pub radii: RadiusTokens,
    pub borders: BorderTokens,
    pub elevation: ElevationTokens,
    pub motion: MotionTokens,
}

impl ThemeDefinition {
    pub fn resolve(
        &self,
        controller: ThemeController,
        preferences: UserPreferences,
    ) -> ResolvedTheme {
        let scheme = controller.effective_scheme();
        ResolvedTheme {
            scheme,
            colors: self.colors.resolve(scheme, preferences),
            typography: self.typography.resolved(preferences),
            spacing: self.spacing,
            radii: self.radii,
            borders: self.borders,
            elevation: self.elevation,
            motion: self.motion.resolved(preferences),
        }
    }
}

fn normalize_channel(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_signed(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(value: f32) -> f32 {
    let value = normalize_channel(value);
    if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        BorderTokens, ColorScheme, ContrastThemePairs, LinearColor, MINIMUM_TEXT_CONTRAST,
        MINIMUM_UI_CONTRAST, MotionTokens, RadiusTokens, SemanticColorToken, SemanticColors,
        ShadowToken, SpacingTokens, ThemeController, ThemeDefinition, ThemeMode, ThemePair,
        TypographyToken,
    };
    use crate::{ContrastPreference, MotionPreference, UserPreferences};

    fn fixture() -> SemanticColors {
        let colors = (0..13)
            .map(|index| LinearColor::new(index as f32 / 13.0, 0.0, 0.0, 1.0))
            .collect::<Vec<_>>();
        SemanticColors {
            background: colors[0],
            surface: colors[1],
            surface_elevated: colors[2],
            text: colors[3],
            text_muted: colors[4],
            primary: colors[5],
            on_primary: colors[6],
            border: colors[7],
            focus: colors[8],
            error: colors[9],
            on_error: colors[10],
            success: colors[11],
            warning: colors[12],
        }
    }

    #[test]
    fn color_channels_are_finite_and_bounded() {
        assert_eq!(
            LinearColor::new(-1.0, 2.0, f32::NAN, 0.5).to_array(),
            [0.0, 1.0, 0.0, 0.5]
        );
    }

    #[test]
    fn srgb_boundary_conversion_round_trips() {
        let source = [0.0, 0.25, 0.5, 1.0];
        let round_trip =
            LinearColor::from_srgb(source[0], source[1], source[2], source[3]).to_srgb_array();

        for (actual, expected) in round_trip.into_iter().zip(source) {
            assert!((actual - expected).abs() < 0.000_01);
        }
    }

    #[test]
    fn contrast_math_uses_linear_luminance_and_alpha_compositing() {
        let black = LinearColor::from_srgb8(0, 0, 0, 255);
        let white = LinearColor::from_srgb8(255, 255, 255, 255);
        let half_black = LinearColor::from_srgb8(0, 0, 0, 128);
        let composited = half_black.composite_over(white);

        assert!((black.contrast_ratio(white) - 21.0).abs() < 0.000_01);
        assert_eq!(white.contrast_ratio(white), 1.0);
        assert!(composited.relative_luminance() > black.relative_luminance());
        assert!(composited.relative_luminance() < white.relative_luminance());
        assert_eq!(
            LinearColor::default().composite_over(LinearColor::default()),
            LinearColor::default()
        );
    }

    #[test]
    fn every_builtin_palette_meets_text_and_ui_contrast_targets() {
        let variants = ContrastThemePairs::default();
        let palettes = [
            variants.normal.light,
            variants.normal.dark,
            variants.more.light,
            variants.more.dark,
            variants.less.light,
            variants.less.dark,
        ];

        for colors in palettes {
            let text_pairs = [
                (colors.text, colors.background),
                (colors.text, colors.surface),
                (colors.text_muted, colors.background),
                (colors.on_primary, colors.primary),
                (colors.on_error, colors.error),
            ];
            for (foreground, background) in text_pairs {
                assert!(
                    foreground.contrast_ratio(background) >= MINIMUM_TEXT_CONTRAST,
                    "contrast={} foreground={foreground:?} background={background:?}",
                    foreground.contrast_ratio(background)
                );
            }

            for foreground in [colors.primary, colors.focus, colors.error] {
                assert!(
                    foreground.contrast_ratio(colors.background) >= MINIMUM_UI_CONTRAST,
                    "contrast={} foreground={foreground:?} background={:?}",
                    foreground.contrast_ratio(colors.background),
                    colors.background
                );
            }
        }
    }

    #[test]
    fn every_semantic_token_resolves_without_component_specific_colors() {
        let colors = fixture();
        let tokens = [
            SemanticColorToken::Background,
            SemanticColorToken::Surface,
            SemanticColorToken::SurfaceElevated,
            SemanticColorToken::Text,
            SemanticColorToken::TextMuted,
            SemanticColorToken::Primary,
            SemanticColorToken::OnPrimary,
            SemanticColorToken::Border,
            SemanticColorToken::Focus,
            SemanticColorToken::Error,
            SemanticColorToken::OnError,
            SemanticColorToken::Success,
            SemanticColorToken::Warning,
        ];

        for (index, token) in tokens.into_iter().enumerate() {
            assert_eq!(colors.resolve(token).red, index as f32 / 13.0);
        }
    }

    #[test]
    fn dimensional_tokens_normalize_invalid_geometry() {
        assert_eq!(
            SpacingTokens::new([-1.0, 4.0, f32::NAN, 16.0, 24.0]),
            SpacingTokens {
                extra_small: 0.0,
                small: 4.0,
                medium: 0.0,
                large: 16.0,
                extra_large: 24.0,
            }
        );
        assert_eq!(RadiusTokens::new(-1.0, 6.0, 12.0, 999.0).small, 0.0);
        assert_eq!(BorderTokens::new(1.0, f32::INFINITY, 3.0).regular, 0.0);
    }

    #[test]
    fn typography_enforces_valid_weight_metrics_and_platform_scale() {
        let token = TypographyToken::new("Vazirmatn", 16.0, 2_000, 12.0, -0.25);
        let preferences = UserPreferences::new(
            MotionPreference::NoPreference,
            ContrastPreference::NoPreference,
            1.5,
        );

        assert_eq!(token.weight, 1_000);
        assert_eq!(token.line_height, 16.0);
        assert_eq!(token.letter_spacing, -0.25);
        assert_eq!(token.resolved_size(preferences), 24.0);
        assert_eq!(token.resolved(preferences).line_height, 24.0);
        assert_eq!(token.resolved(preferences).letter_spacing, -0.375);
    }

    #[test]
    fn shadows_keep_signed_offsets_and_normalize_blur() {
        let shadow = ShadowToken::new(-2.0, 4.0, -8.0, -1.0, LinearColor::new(0.0, 0.0, 0.0, 0.2));

        assert_eq!(shadow.offset_x, -2.0);
        assert_eq!(shadow.offset_y, 4.0);
        assert_eq!(shadow.blur, 0.0);
        assert_eq!(shadow.spread, -1.0);
    }

    #[test]
    fn motion_tokens_resolve_reduced_motion_centrally() {
        let motion = MotionTokens {
            fast: Duration::from_millis(80),
            normal: Duration::from_millis(160),
            slow: Duration::from_millis(280),
        };
        let reduced = UserPreferences::new(
            MotionPreference::Reduce,
            ContrastPreference::NoPreference,
            1.0,
        );

        assert_eq!(motion.resolved(reduced), MotionTokens::default());
    }

    #[test]
    fn built_in_light_and_dark_palettes_use_semantic_amber_primary() {
        let themes = ThemePair::default();

        assert_eq!(
            themes.light.primary.to_srgb_array(),
            LinearColor::from_srgb8(180, 83, 9, 255).to_srgb_array()
        );
        assert_eq!(
            themes.dark.primary.to_srgb_array(),
            LinearColor::from_srgb8(245, 158, 11, 255).to_srgb_array()
        );
        assert_ne!(themes.light.background, themes.dark.background);
        assert_ne!(themes.light.text, themes.dark.text);
    }

    #[test]
    fn runtime_theme_switching_tracks_only_effective_changes() {
        let mut controller = ThemeController::default();
        assert_eq!(controller.mode(), ThemeMode::System);
        assert_eq!(controller.effective_scheme(), ColorScheme::Light);
        assert_eq!(controller.generation(), 0);

        assert!(!controller.set_mode(ThemeMode::Light));
        assert_eq!(controller.generation(), 0);
        assert!(controller.set_mode(ThemeMode::Dark));
        assert_eq!(controller.effective_scheme(), ColorScheme::Dark);
        assert_eq!(controller.generation(), 1);
        assert!(!controller.set_system_scheme(ColorScheme::Dark));
        assert!(!controller.set_mode(ThemeMode::System));
        assert_eq!(controller.generation(), 1);
        assert!(controller.set_system_scheme(ColorScheme::Light));
        assert_eq!(controller.generation(), 2);
    }

    #[test]
    fn controller_resolves_active_palette_without_mutating_theme_data() {
        let themes = ThemePair::default();
        let mut controller = ThemeController::default();
        assert_eq!(controller.colors(themes), themes.light);

        controller.set_mode(ThemeMode::Dark);
        assert_eq!(controller.colors(themes), themes.dark);
    }

    #[test]
    fn contrast_preference_selects_the_matching_scheme_palette() {
        let themes = ContrastThemePairs::default();
        let normal = UserPreferences::default();
        let more = UserPreferences::new(
            MotionPreference::NoPreference,
            ContrastPreference::More,
            1.0,
        );
        let less = UserPreferences::new(
            MotionPreference::NoPreference,
            ContrastPreference::Less,
            1.0,
        );

        assert_eq!(
            themes.resolve(ColorScheme::Dark, normal),
            themes.normal.dark
        );
        assert_eq!(themes.resolve(ColorScheme::Dark, more), themes.more.dark);
        assert_eq!(themes.resolve(ColorScheme::Light, less), themes.less.light);
    }

    #[test]
    fn resolved_theme_applies_all_preferences_once_for_widget_consumers() {
        let definition = ThemeDefinition::default();
        let mut controller = ThemeController::default();
        controller.set_mode(ThemeMode::Dark);
        let preferences =
            UserPreferences::new(MotionPreference::Reduce, ContrastPreference::More, 1.5);

        let resolved = definition.resolve(controller, preferences);

        assert_eq!(resolved.scheme, ColorScheme::Dark);
        assert_eq!(resolved.colors, definition.colors.more.dark);
        assert_eq!(resolved.typography.size, 24.0);
        assert_eq!(resolved.typography.line_height, 36.0);
        assert_eq!(resolved.motion, MotionTokens::default());
    }
}
